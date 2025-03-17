use crate::args::CLI_OPTS;
use crate::file::FileOps;
use anyhow::Result;
use log::{debug, error, trace};
use std::collections::{HashMap, HashSet};
use tokio::fs::{canonicalize, metadata};
use walkdir::WalkDir;

pub async fn size_mode() -> Result<(usize, usize)> {
    let local_path = &CLI_OPTS.local_path;
    let remote_path = &CLI_OPTS.remote_path.as_ref().expect("Remote Path is None");
    let mut file_map = HashMap::new();
    let (mut num_processed, mut num_duplicates) = (0, 0);

    if canonicalize(remote_path).await? == canonicalize(local_path).await? {
        anyhow::bail!(
            "In-place deduplication not yet supported. {} and {} are the same path.",
            remote_path.display(),
            local_path.display()
        );
    }

    for entry in WalkDir::new(remote_path) {
        let remote_file = match entry {
            Ok(file) => file.into_path(),
            Err(e) => {
                error!("Error while walking {}: {}", remote_path.display(), e);
                continue;
            }
        };

        if remote_file.is_dir() {
            continue;
        }

        let size = metadata(&remote_file).await?.len();
        file_map
            .entry(size)
            .or_insert(HashSet::new())
            .insert(remote_file);
    }

    for entry in WalkDir::new(local_path) {
        let local_file = match entry {
            Ok(file) => file.into_path(),
            Err(e) => {
                error!("Error while walking {}: {}", local_path.display(), e);
                continue;
            }
        };

        if local_file.is_dir() {
            continue;
        }

        num_processed += 1;
        let size = metadata(&local_file).await?.len();
        if !file_map.contains_key(&size) {
            continue;
        }

        if !file_map[&size].is_empty() {
            trace!(
                "Found multiple of same size={size}: {}, {:?}",
                local_file.display(),
                file_map[&size]
            );
            let local_chksum = local_file.content_checksum::<sha1::Sha1>().await?;
            for remote_file in &file_map[&size] {
                let remote_chksum = remote_file.content_checksum::<sha1::Sha1>().await?;
                if local_chksum == remote_chksum {
                    let action = if CLI_OPTS.commit { "remov" } else { "process" };
                    debug!(
                        "{action}ing duplicate files: local={} remote={}",
                        local_file.display(),
                        remote_file.display()
                    );
                    if let Err(e) = local_file.remove_file(CLI_OPTS.commit).await {
                        error!("Error {action}ing file {}: {e}", local_file.display());
                    }
                    num_duplicates += 1;
                    break;
                }
            }
        }
    }

    Ok((num_processed, num_duplicates))
}
