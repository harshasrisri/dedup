use crate::file::FileOps;
use anyhow::{Error, Result};
use log::{debug, error, info};
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;
use std::str::FromStr;
use std::{collections::HashSet, fmt::Display};
use tokio::fs::metadata;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub enum DigestKind {
    MD5,
    SHA1,
    SHA2,
}

impl FromStr for DigestKind {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let digest = s.to_ascii_lowercase();
        match digest.as_str() {
            "md5" => Ok(DigestKind::MD5),
            "sha1" | "sha128" => Ok(DigestKind::SHA1),
            "sha2" | "sha256" => Ok(DigestKind::SHA2),
            _ => Err(format!("Unsupported/Invalid digest algorithm: {digest}")),
        }
    }
}

impl Display for DigestKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DigestKind::MD5 => "MD5",
            DigestKind::SHA1 => "SHA1",
            DigestKind::SHA2 => "SHA2",
        })
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

pub async fn digest_mode<P: AsRef<Path>>(
    local_path: P,
    remote_list: P,
    digest: &DigestKind,
    commit: bool,
) -> Result<(usize, usize)> {
    if !local_path.as_ref().exists() {
        anyhow::bail!("Local path not found - {}", local_path.as_ref().display());
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
                error!(
                    "{}: error while walking: {e}",
                    local_path.as_ref().display()
                );
                continue;
            }
        };

        if file_path.is_dir() {
            continue;
        }

        num_processed += 1;

        let action = if commit { "remov" } else { "process" };
        let chksum = file_path.digest(digest).await?;

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

pub async fn analyze_path<P: AsRef<Path>>(
    local_path: P,
    digest: &DigestKind,
    output_file: P,
) -> Result<()> {
    if !local_path.as_ref().exists() {
        anyhow::bail!("Local path not found - {}", local_path.as_ref().display());
    }

    let mut file_map = HashMap::new();
    let mut num_analyzed = 0;

    for entry in WalkDir::new(&local_path) {
        let file_path = match entry {
            Ok(file) => file.into_path(),
            Err(e) => {
                error!(
                    "{}: error while walking: {e}",
                    local_path.as_ref().display()
                );
                continue;
            }
        };

        if file_path.is_dir() {
            continue;
        }

        num_analyzed += 1;
        let chksum = file_path.digest(digest).await?;
        let size = metadata(&file_path).await?.len();

        file_map
            .entry(size)
            .or_insert(HashSet::new())
            .insert(chksum);
    }

    info!("Analyzed {num_analyzed} files");
    write_output(&file_map, &output_file).await
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

    Ok(())
}
