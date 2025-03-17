use crate::args::CLI_OPTS;
use crate::file::FileOps;
use anyhow::anyhow;
use anyhow::{Error, Result};
use log::{debug, error};
use std::collections::HashSet;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use walkdir::WalkDir;

pub async fn checksum<P: AsRef<Path>>(path: &P) -> Result<String> {
    let algo = CLI_OPTS
        .hash
        .as_ref()
        .ok_or(Error::msg("Hash algo should have been set"))?;
    match algo.as_str() {
        "MD5" | "Md5" | "md5" => path.content_digest::<md5::Md5>().await,
        "SHA1" | "Sha1" | "sha1" => path.content_digest::<sha1::Sha1>().await,
        "SHA2" | "Sha2" | "sha2" => path.content_digest::<sha2::Sha256>().await,
        "SHA128" | "Sha128" | "sha128" => path.content_digest::<sha1::Sha1>().await,
        "SHA256" | "Sha256" | "sha256" => path.content_digest::<sha2::Sha256>().await,
        "SHA512" | "Sha512" | "sha512" => path.content_digest::<sha2::Sha512>().await,
        _ => anyhow::bail!("Unsupported hash algorithm - {}", algo),
    }
}

async fn dedup_from_set<P: AsRef<Path>>(filepath: &P, checksums: &HashSet<String>) -> Result<bool> {
    let chksum = checksum(&filepath).await?;
    if checksums.contains(&chksum) {
        let action = if CLI_OPTS.commit { "remov" } else { "process" };
        debug!("{action}ing {}", filepath.as_ref().display());
        if let Err(e) = filepath.remove_file(CLI_OPTS.commit).await {
            error!(
                "Error {action}ing file {}: {e}",
                filepath.as_ref().display()
            );
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn remote_chksums() -> Result<HashSet<String>> {
    let filepath = CLI_OPTS
        .remote_list
        .as_ref()
        .ok_or_else(|| anyhow!("Should have had a remote list to work with"))?;
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
        ret.insert(line.to_string());
    }
    Ok(ret)
}

pub async fn hash_mode() -> Result<(usize, usize)> {
    let local_path = &CLI_OPTS.local_path;
    if !local_path.exists() {
        anyhow::bail!("Local path not found - {}", local_path.to_str().unwrap());
    }

    let checksums = remote_chksums().await?;
    let (mut num_processed, mut num_duplicates) = (0, 0);

    for entry in WalkDir::new(local_path) {
        let file_path = match entry {
            Ok(file) => file.into_path(),
            Err(e) => {
                error!("Error while walking {}: {}", local_path.display(), e);
                continue;
            }
        };

        if file_path.is_dir() {
            continue;
        }

        num_processed += 1;
        num_duplicates += dedup_from_set(&file_path, &checksums)
            .await
            .unwrap_or(false) as usize;
    }

    Ok((num_processed, num_duplicates))
}
