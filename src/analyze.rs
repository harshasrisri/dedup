use crate::digest::DigestKind;
use crate::file::FileOps;
use anyhow::Result;
use clap::Args;
use log::{debug, error, info};
use std::fmt::Write;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};
use tokio::fs::{canonicalize, metadata};
use tokio::io::{AsyncWriteExt, BufWriter};
use walkdir::WalkDir;

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
/// Analyzes path and writes data to file to be used elsewhere
pub struct Analyze {
    /// File to write the output of digest-mode analysis
    #[arg(short, long, requires = "digest", default_value = "dedup.out")]
    pub output_file: PathBuf,

    /// Local Path containing files that need to be checked for duplicates
    #[arg(short, long, default_value = ".")]
    pub local_path: PathBuf,

    /// Type of digest to use to parse/generate digest-mode analysis
    #[arg(short, long, default_value = "sha1")]
    pub digest: DigestKind,
}

impl Analyze {
    pub async fn analyze(&self) -> Result<()> {
        debug!(
            "Starting digest mode analysis at {}, using {} and writing out to {}",
            canonicalize(&self.local_path).await.unwrap().display(),
            self.digest,
            self.output_file.display()
        );

        if !self.local_path.exists() {
            anyhow::bail!("Local path not found - {}", self.local_path.display());
        }

        let mut file_map = HashMap::new();
        let mut num_analyzed = 0;

        for entry in WalkDir::new(&self.local_path) {
            let file_path = match entry {
                Ok(file) => file.into_path(),
                Err(e) => {
                    error!("{}: error while walking: {e}", self.local_path.display());
                    continue;
                }
            };

            if file_path.is_dir() || file_path.is_symlink() {
                continue;
            }

            num_analyzed += 1;
            let chksum = file_path.digest(&self.digest).await?;
            let size = metadata(&file_path).await?.len();

            file_map
                .entry(size)
                .or_insert(HashSet::new())
                .insert(chksum);
        }

        info!("Analyzed {num_analyzed} files");
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
