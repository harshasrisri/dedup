use crate::{digest::DigestKind, file::FileOps};
use anyhow::{Error, Result};
use clap::Args;
use log::{debug, error, info};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};
use tokio::{
    fs::metadata,
    io::{AsyncBufReadExt, BufReader},
};
use walkdir::WalkDir;

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
/// Uses analysis from a file to dedup files
pub struct Remote {
    /// File containing digest-mode analysis used to dedup files in local_path
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

        let analysis = parse_input(&self.input_file.as_ref().unwrap()).await?;
        info!(
            "Found {} entries in input file {}",
            analysis.len(),
            self.input_file.as_ref().unwrap().display()
        );
        let (mut num_processed, mut num_duplicates) = (0, 0);

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

            num_processed += 1;

            let size = metadata(&file_path).await?.len();

            if let Some(digests) = analysis.get(&size) {
                let digest = file_path.digest(&self.digest).await?;
                if !digests.contains(&digest) {
                    debug!("skipping file: {}", file_path.display());
                    continue;
                }
            } else {
                debug!("skipping file: {}", file_path.display());
                continue;
            }

            num_duplicates += 1;
            info!("duplicate file: {}", file_path.display());
            if let Err(e) = file_path.remove_file(commit).await {
                let action = if commit { "remov" } else { "process" };
                error!("error {action}ing file {}: {e}", file_path.display());
            }
        }

        Ok((num_processed, num_duplicates))
    }
}

async fn parse_input<P: AsRef<Path>>(input_file: P) -> Result<HashMap<u64, HashSet<String>>> {
    let filepath = input_file.as_ref();
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
        let Some((size, digests)) = line.split_once(":") else {
            anyhow::bail!("Failed to parse input line {line}");
        };
        let digests = digests
            .split(",")
            .map(|s| s.trim().to_string())
            .collect::<HashSet<String>>();
        ret.insert(size.trim().parse()?, digests);
    }
    Ok(ret)
}
