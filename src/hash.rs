use crate::file::FileOps;
use anyhow::{Error, Result};
use log::{debug, error, info};
use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, BufReader};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub enum HashMode {
    MD5,
    SHA1,
    SHA2,
}

impl FromStr for HashMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let hash = s.to_ascii_lowercase();
        match hash.as_str() {
            "md5" => Ok(HashMode::MD5),
            "sha1" | "sha128" => Ok(HashMode::SHA1),
            "sha2" | "sha256" => Ok(HashMode::SHA2),
            _ => Err(format!("Unsupported/Invalid hash algorithm: {hash}")),
        }
    }
}

impl HashMode {
    pub async fn digest_file<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        match &self {
            HashMode::MD5 => path.content_digest::<md5::Md5>().await,
            HashMode::SHA1 => path.content_digest::<sha1::Sha1>().await,
            HashMode::SHA2 => path.content_digest::<sha2::Sha256>().await,
        }
    }
}

async fn remote_chksums<P: AsRef<Path>>(remote_list: P) -> Result<HashSet<String>> {
    let filepath = remote_list.as_ref();
    let remote: Box<dyn tokio::io::AsyncRead + Unpin> = match filepath
        .to_str()
        .ok_or(Error::msg("Error reading filename"))?
    {
        "-" => Box::new(tokio::io::stdin()),
        path => Box::new(path.open_ro().await?),
    };

    let mut lines = BufReader::new(remote).lines();
    let mut ret = HashSet::new();
    while let Some(line) = lines.next_line().await? {
        let line = line.split_whitespace().next().unwrap_or_default();
        ret.insert(line.trim().to_string());
    }
    Ok(ret)
}

pub async fn hash_mode<P: AsRef<Path>>(
    local_path: P,
    remote_list: P,
    hash: &HashMode,
    commit: bool,
) -> Result<(usize, usize)> {
    let local_path = local_path;
    if !local_path.as_ref().exists() {
        anyhow::bail!(
            "Local path not found - {}",
            local_path.as_ref().to_str().unwrap()
        );
    }

    let checksums = remote_chksums(&remote_list).await?;
    info!(
        "Found {} entries in remote list {}",
        checksums.len(),
        remote_list.as_ref().display()
    );
    let (mut num_processed, mut num_duplicates) = (0, 0);

    for entry in WalkDir::new(&local_path) {
        let file_path = match entry {
            Ok(file) => file.into_path(),
            Err(e) => {
                error!("{}: error while walking: {e}", local_path.as_ref().display());
                continue;
            }
        };

        if file_path.is_dir() {
            continue;
        }

        num_processed += 1;

        let action = if commit { "remov" } else { "process" };
        let chksum = hash.digest_file(&file_path).await?;

        if !checksums.contains(&chksum) {
            debug!("skipping file: {}", file_path.display());
            continue;
        }

        num_duplicates += 1;
        info!("{action}ing file: {}", file_path.display());
        if let Err(e) = file_path.remove_file(commit).await {
            error!("error {action}ing file {}: {e}", file_path.display());
        }
    }

    Ok((num_processed, num_duplicates))
}
