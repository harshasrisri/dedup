use crate::args::CLI_OPTS;
use crate::file::FileOps;
use md5::Md5;
use sha1::Sha1;
use sha2::Sha256;
use sha2::Sha512;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::Result;

pub fn checksum(path: &Path) -> Result<String> {
    let algo = CLI_OPTS
        .hash
        .as_ref()
        .expect("Hash algo should have been set");
    match algo.as_str() {
        "MD5" | "Md5" | "md5" => path.content_checksum::<Md5>(),
        "SHA128" | "Sha128" | "sha128" => path.content_checksum::<Sha1>(),
        "SHA256" | "Sha256" | "sha256" => path.content_checksum::<Sha256>(),
        "SHA512" | "Sha512" | "sha512" => path.content_checksum::<Sha512>(),
        _ => panic!("Unsupported hash algorithm - {}", algo),
    }
}

fn dedup_from_set(filepath: &Path, checksums: &HashSet<String>) -> Result<bool> {
    match checksum(&filepath) {
        Ok(chksum) => {
            if checksums.contains(&chksum) {
                filepath.remove_file()?;
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(e) => {
            eprintln!(
                "Skipping {} - {}",
                &filepath.to_str().unwrap(),
                e.to_string()
            );
            Ok(false)
        }
    }
}

fn list_file_to_set(filepath: &PathBuf) -> Result<HashSet<String>> {
    let remote: Box<dyn std::io::Read> = match filepath.to_str().unwrap() {
        "-" => Box::new(std::io::stdin()),
        _ => Box::new(File::open(&filepath.to_str().unwrap())?),
    };
    Ok(BufReader::new(remote)
        .lines()
        .filter_map(|result| result.ok())
        .filter_map(|line| line.split(' ').nth(0).map(|slice| slice.to_string()))
        .map(|hash| hash.to_owned())
        .collect())
}

pub fn hash_mode(list: &PathBuf) -> Result<()> {
    let mut total_count: usize = 0;
    let mut dup_count: usize = 0;

    let local = &CLI_OPTS.local_path;
    if !local.exists() {
        anyhow::bail!("Local path not found - {}", local.to_str().unwrap());
    }
    let checksums = list_file_to_set(&list).expect("Failed to parse remote list");

    for file in WalkDir::new(&local) {
        let file = file.unwrap();
        if file.path().is_dir() {
            continue;
        }
        total_count += 1;
        dup_count += dedup_from_set(&file.path(), &checksums)? as usize;
    }

    println!(
        "{} files processed. {} Duplicates {}",
        total_count,
        dup_count,
        if CLI_OPTS.commit {
            "deleted".to_string()
        } else {
            "found".to_string()
        }
    );
    Ok(())
}
