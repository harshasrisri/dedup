use crate::args::CLI_OPTS;
use crate::file::FileOps;
use std::collections::HashMap;
use std::fs::metadata;
use walkdir::WalkDir;
use anyhow::Result;

pub fn size_mode() -> Result<()> {
    let ref local_path = CLI_OPTS.local_path;
    let ref remote_path = CLI_OPTS.remote_path.as_ref().expect("Remote Path is None");
    let mut file_map = HashMap::new();
    let ( mut processed, mut duplicates ) = (0, 0);

    for file in WalkDir::new(remote_path) {
        let path = file?.into_path();
        if path.is_dir() {
            continue;
        }
        let size = metadata(&path)?.len();
        file_map.entry(size).or_insert(Vec::new()).push(path);
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
        for src in file_map[&size].iter() {
            if src.content_equals(&path)? {
                path.remove_file()?;
                duplicates += 1;
            }
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
