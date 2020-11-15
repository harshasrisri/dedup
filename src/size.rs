use crate::args::CLI_OPTS;
use crate::file::{files_are_equal, remove_file};
use std::collections::HashMap;
use std::fs::metadata;
use walkdir::WalkDir;

pub fn size_mode() -> Result<(), Box<dyn std::error::Error>> {
    let ref local_path = CLI_OPTS.local_path;
    let ref remote_path = CLI_OPTS.remote_path.as_ref().expect("Remote Path is None");
    let mut file_map = HashMap::new();

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
        let size = metadata(&path)?.len();
        if !file_map.contains_key(&size) {
            continue;
        }
        for src in file_map[&size].iter() {
            if files_are_equal(&src, &path)? {
                remove_file(&path)?;
            }
        }
    }

    Ok(())
}
