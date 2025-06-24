use anyhow::Result;
use clap::Args;
use log::{debug, error, trace};
use tokio::fs::{canonicalize, metadata};
use walkdir::WalkDir;
use std::{collections::{HashMap, HashSet}, path::PathBuf};

use crate::digest::DigestKind;

#[derive(Args, Debug)]
#[command(arg_required_else_help = true)]
/// Dedups files in one folder while referencing another folder
pub struct Local {
    /// Path to use as a reference to filter duplicates in local
    #[arg(short, long, conflicts_with = "input_file")]
    pub reference_path: Option<PathBuf>,

    /// Local Path containing files that need to be checked for duplicates
    #[arg(short, long, default_value = ".")]
    pub local_path: PathBuf,
}

impl Local {
    pub async fn dedup(&self, commit: bool) -> Result<(usize, usize)> {
        debug!(
            "Starting size mode dedup as {} using remote path {}",
            self.local_path.display(),
            self.reference_path.as_ref().unwrap().display()
        );
        let remote_path = self.reference_path.as_ref().unwrap();
        let mut file_map = HashMap::new();
        let (mut num_processed, mut num_duplicates) = (0, 0);

        if canonicalize(&remote_path).await? == canonicalize(&self.local_path).await? {
            anyhow::bail!(
                "In-place deduplication not yet supported. {} and {} are the same path.",
                remote_path.display(),
                self.local_path.display()
            );
        }

        for entry in WalkDir::new(&remote_path) {
            let remote_file = match entry {
                Ok(file) => file.into_path(),
                Err(e) => {
                    error!(
                        "Error while walking {}: {}",
                        remote_path.display(),
                        e
                    );
                    continue;
                }
            };

            if remote_file.is_dir() || remote_file.is_symlink() {
                continue;
            }

            let size = metadata(&remote_file).await?.len();
            file_map
                .entry(size)
                .or_insert(HashSet::new())
                .insert(remote_file);
        }

        for entry in WalkDir::new(&self.local_path) {
            let local_file = match entry {
                Ok(file) => file.into_path(),
                Err(e) => {
                    error!(
                        "Error while walking {}: {}",
                        self.local_path.display(),
                        e
                    );
                    continue;
                }
            };

            if local_file.is_dir() || local_file.is_symlink() {
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
                let local_chksum = local_file.digest(&DigestKind::SHA1).await?;
                for remote_file in &file_map[&size] {
                    let remote_chksum = remote_file.digest(&DigestKind::SHA1).await?;
                    if local_chksum == remote_chksum {
                        let action = if commit { "removing" } else { "found" };
                        debug!(
                            "{action} duplicate files: local={} remote={}",
                            local_file.display(),
                            remote_file.display()
                        );
                        if let Err(e) = local_file.remove_file(commit).await {
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
}
