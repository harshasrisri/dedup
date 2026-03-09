use crate::hasher::HashFile;
use crate::file::DirOps;
use crate::file::FileOps;
use anyhow::Result;
use clap::Args;
use futures::{StreamExt, stream};
use log::{debug, error, info};
use std::fmt::Write;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};
use tokio::fs::{canonicalize, metadata};
use tokio::io::{AsyncWriteExt, BufWriter};

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
/// Analyzes path and writes data to file to be used elsewhere
pub struct Analyze {
    /// File to write the output of hash analysis
    #[arg(short, long, default_value = "dedup.out")]
    pub output_file: PathBuf,

    /// Local Path containing files that need to be checked for duplicates
    #[arg(short, long, default_value = ".")]
    pub local_path: PathBuf,
}

impl Analyze {
    pub async fn analyze(&self) -> Result<()> {
        debug!(
            "Starting analysis at {}, and writing out to {}",
            canonicalize(&self.local_path).await.unwrap().display(),
            self.output_file.display()
        );

        if !self.local_path.exists() {
            anyhow::bail!("Local path not found - {}", self.local_path.display());
        }

        let mut file_map = HashMap::new();
        let mut num_analyzed = 0;

        let entries = self.local_path.walkdir();

        let mut stream = stream::iter(entries)
            .map(move |file_path| {
                async move {
                    debug!("Start analyzing file: {}", file_path.display());
                    match async {
                        let chksum = file_path.chksum()?;
                        let size = metadata(&file_path).await?.len();
                        debug!("Finished analyzing file: {}", file_path.display());
                        Ok::<_, Box<dyn std::error::Error>>((size, chksum))
                    }
                    .await
                    {
                        Ok((size, chksum)) => Some((size, chksum)),
                        Err(e) => {
                            error!("Error analyzing {}: {e}", file_path.display());
                            None
                        }
                    }
                }
            })
            .buffer_unordered(num_cpus::get() * 4);

        while let Some(Some((size, chksum))) = stream.next().await {
            num_analyzed += 1;
            file_map
                .entry(size)
                .or_insert(HashSet::new())
                .insert(chksum);
        }

        info!("Analyzed {num_analyzed} files, writing to output...");
        write_output(&file_map, &self.output_file).await
    }
}

async fn write_output<P: AsRef<Path>>(
    file_map: &HashMap<u64, HashSet<String>>,
    output_file: &P,
) -> Result<()> {
    let file = output_file.open_rw().await?;
    let mut writer = BufWriter::new(file);

    for (size, list) in file_map {
        let mut buffer = format!("{size}:");
        let mut stream = list.iter();
        if let Some(item) = stream.next() {
            write!(buffer, "{item}").unwrap();
        }
        for item in stream {
            write!(buffer, ",{item}").unwrap();
        }
        writeln!(buffer).unwrap();

        writer.write_all(buffer.as_bytes()).await?;
    }

    writer.flush().await?;

    debug!(
        "Analysis written to file {}",
        output_file.as_ref().display()
    );
    Ok(())
}
