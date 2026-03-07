use crate::{digest::DigestKind, file::DirOps, file::FileOps};
use anyhow::{Error, Result};
use clap::Args;
use futures::{StreamExt, stream};
use log::{debug, error, info};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    fs::metadata,
    io::{AsyncBufReadExt, BufReader},
};

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
/// Uses analysis from a file to dedup files
pub struct Remote {
    /// File containing digest-mode analysis used to dedup files in `local_path`
    #[arg(short, long, requires = "digest")]
    pub input_file: Option<PathBuf>,

    /// Local Path containing files that need to be checked for duplicates
    #[arg(short, long, default_value = ".")]
    pub local_path: PathBuf,

    /// Type of digest to use to parse/generate digest-mode analysis
    #[arg(short, long, default_value = "sha1")]
    pub digest: DigestKind,
}

impl Remote {
    pub async fn dedup(&self, commit: bool) -> Result<(usize, usize)> {
        debug!(
            "Starting digest mode dedup at {} using input file {}",
            self.local_path.display(),
            self.input_file.as_ref().unwrap().display()
        );

        if !self.local_path.exists() {
            anyhow::bail!("Local path not found - {}", self.local_path.display());
        }

        let entries = self.local_path.walkdir();
        let digest = self.digest.clone();

        let mut stream = stream::iter(entries)
            .map(move |file_path| {
                let digest = digest.clone();
                async move {
                    debug!("Start analyzing file: {}", file_path.display());
                    let file_path_clone = file_path.clone();
                    match async {
                        let chksum = file_path.digest(&digest).await?;
                        let size = metadata(&file_path).await?.len();
                        debug!("Finished analyzing file: {}", file_path.display());
                        Ok::<_, Box<dyn std::error::Error>>((size, chksum, file_path))
                    }
                    .await
                    {
                        Ok((size, chksum, file_path)) => Some((size, chksum, file_path)),
                        Err(e) => {
                            error!("Error analyzing {}: {e}", file_path_clone.display());
                            None
                        }
                    }
                }
            })
            .buffer_unordered(num_cpus::get() * 4);

        let (analysis, entries) = parse_input(&self.input_file.as_ref().unwrap()).await?;
        let analysis = Arc::new(analysis);
        info!(
            "Found {} entries in input file {}",
            entries,
            self.input_file.as_ref().unwrap().display()
        );

        let (mut num_processed, mut num_duplicates) = (0, 0); //entries.size_hint().0;
        while let Some(Some((size, chksum, file_path))) = stream.next().await {
            num_processed += 1;
            if let Some(chksums) = analysis.get(&size)
                && chksums.contains(&chksum)
            {
                num_duplicates += 1;
                let action = if commit { "remov" } else { "process" };
                if let Err(e) = file_path.remove_file(commit).await {
                    error!("error {action}ing file {}: {e}", file_path.display());
                } else {
                    debug!("successfully {action}ed file {}", file_path.display());
                }
            } else {
                debug!("skipping file: {}", file_path.display());
            }
        }

        Ok((num_processed, num_duplicates))
    }
}

async fn parse_input<P: AsRef<Path>>(
    input_file: P,
) -> Result<(HashMap<u64, HashSet<String>>, usize)> {
    let filepath = input_file.as_ref();
    let mut entry_count = 0;
    let reader: Box<dyn tokio::io::AsyncRead + Unpin> = match filepath
        .to_str()
        .ok_or(Error::msg("Error reading filename"))?
    {
        "-" => Box::new(tokio::io::stdin()),
        path => Box::new(path.open_ro().await?),
    };

    let mut lines = BufReader::new(reader).lines();
    let mut ret = HashMap::new();
    while let Some(line) = lines.next_line().await? {
        let Some((size, digests)) = line.split_once(':') else {
            anyhow::bail!("Failed to parse input line {line}");
        };
        let digests = digests
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<HashSet<String>>();
        entry_count += digests.len();
        ret.insert(size.trim().parse()?, digests);
    }
    Ok((ret, entry_count))
}
