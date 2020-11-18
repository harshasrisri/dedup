use crate::args::CLI_OPTS;
use crate::file::FileOps;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fs::metadata;
use walkdir::WalkDir;

pub fn size_mode() -> Result<()> {
    let ref local_path = CLI_OPTS.local_path;
    let ref remote_path = CLI_OPTS.remote_path.as_ref().expect("Remote Path is None");
    let mut file_map = HashMap::new();
    let (mut processed, mut duplicates) = (0, 0);

    for file in WalkDir::new(remote_path) {
        let path = file?.into_path();
        if path.is_dir() {
            continue;
        }
        let size = metadata(&path)?.len();
        file_map
            .entry(size)
            .or_insert(HashSet::new())
            .insert(path.content_checksum::<sha1::Sha1>()?);
    }

    for file in WalkDir::new(local_path) {
        let path = file?.into_path();
        if path.is_dir() {
            continue;
        }
        processed += 1;
        let size = metadata(&path)?.len();
        if !file_map.contains_key(&size) {
            continue;
        }
        let chksum = path.content_checksum::<sha1::Sha1>()?;

        if file_map[&size].contains(&chksum) {
            path.remove_file()?;
            duplicates += 1;
        }
    }

    println!(
        "{} files processed. {} Duplicates {}",
        processed,
        duplicates,
        if CLI_OPTS.commit {
            "deleted".to_string()
        } else {
            "found".to_string()
        }
    );

    Ok(())
}
