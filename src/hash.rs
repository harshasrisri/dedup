use crate::args::CLI_OPTS;
use crate::file::FileOps;
use anyhow::{Error, Result};
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn checksum<P: AsRef<Path>>(path: &P) -> Result<String> {
    let algo = CLI_OPTS
        .hash
        .as_ref()
        .ok_or(Error::msg("Hash algo should have been set"))?;
    match algo.as_str() {
        "MD5" | "Md5" | "md5" => path.content_checksum::<md5::Md5>(),
        "SHA128" | "Sha128" | "sha128" => path.content_checksum::<sha1::Sha1>(),
        "SHA256" | "Sha256" | "sha256" => path.content_checksum::<sha2::Sha256>(),
        "SHA512" | "Sha512" | "sha512" => path.content_checksum::<sha2::Sha512>(),
        _ => anyhow::bail!("Unsupported hash algorithm - {}", algo),
    }
}

fn dedup_from_set<P: AsRef<Path>>(filepath: &P, checksums: &HashSet<String>) -> Result<bool> {
    let chksum = checksum(&filepath)?;
    if checksums.contains(&chksum) {
        return filepath.remove_file().map(|_| true);
    }
    Ok(false)
}

fn list_file_to_set<P: AsRef<Path>>(filepath: &P) -> Result<HashSet<String>> {
    let remote: Box<dyn std::io::Read> = match filepath
        .as_ref()
        .to_str()
        .ok_or(Error::msg("Error reading filename"))?
    {
        "-" => Box::new(std::io::stdin()),
        _ => Box::new(filepath.open_ro()?),
    };
    Ok(BufReader::new(remote)
        .lines()
        .filter_map(|result| result.ok())
        .filter_map(|line| line.split(' ').nth(0).map(|slice| slice.to_string()))
        .map(|hash| hash.to_owned())
        .collect())
}

pub fn hash_mode<P: AsRef<Path>>(list: &P) -> Result<()> {
    let mut total_count: usize = 0;
    let mut dup_count: usize = 0;

    let local = &CLI_OPTS.local_path;
    if !local.exists() {
        anyhow::bail!("Local path not found - {}", local.to_str().unwrap());
    }
    let checksums = list_file_to_set(&list)?;

    for file in walkdir::WalkDir::new(&local) {
        let file = file.unwrap();
        if file.path().is_dir() {
            continue;
        }
        total_count += 1;
        dup_count += dedup_from_set(&file.path(), &checksums).unwrap_or(false) as usize;
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
