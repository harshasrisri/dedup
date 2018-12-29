#[macro_use]
extern crate structopt;
extern crate md5;
extern crate sha1;
extern crate sha2;
extern crate digest;
extern crate ignore;

use std::io::Result;
use std::fs::File;
use std::path::Path;
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use ignore::WalkBuilder;
use structopt::StructOpt;

mod args;
use args::DedupOpts;

mod hash;
use hash::checksum;

fn remove_file (filepath: &Path) -> Result<()> {
    if DedupOpts::from_args().verbose > 0 {
        println!("{}", &filepath.to_str().unwrap());
    }

    if DedupOpts::from_args().commit == true {
        std::fs::remove_file(filepath)?;
    }

    Ok(())
}

fn dedup_from_set (filepath : &Path, checksums : &HashSet<String>) -> u8 {
    if filepath.is_dir() == true {
        return 0;
    }

    let chksum = checksum(&filepath).expect(&format!("Error calculating checksum of {}", &filepath.to_str().unwrap()));

    if checksums.contains(&chksum) {
        let _ = remove_file(filepath);
        return 1;
    }
    0
}

fn list_file_to_set (filepath : &Path) -> Result<HashSet<String>> {
    let mut checksums = HashSet::new();
    let remote = File::open(&filepath.to_str().unwrap())?;
    for line in BufReader::new(remote).lines().filter_map(|result| result.ok()) {
        let hashpath : Vec<&str> = line.splitn(2, ' ').collect();
        checksums.insert(hashpath[0].to_string());
    }
    Ok(checksums)
}

fn remote_dir_to_set (dirpath : &Path) -> Result<HashSet<String>> {
    let mut checksums = HashSet::new();
    for path in WalkBuilder::new(&dirpath).hidden(false).build().filter_map(|x| x.ok()) {
        if path.path().is_dir() == false {
            checksums.insert(checksum(&path.path())?);
        }
    }
    Ok(checksums)
}

fn main () {
    let args = DedupOpts::from_args();
    if args.debug == true {
        println!("{:?}", args);
    }

    let local = &args.local_path;
    if local.exists() == false {
        println!("Local path not found - {}", local.to_str().unwrap());
        return
    }

    let checksums = match (&args.remote_list, &args.remote_path) {
        (Some(_),    Some(_)   ) => panic!("StructOpt option parsing should have prevented this"),
        (Some(list), None      ) => list_file_to_set(list),
        (None,       Some(path)) => remote_dir_to_set(path),
        (None,       None      ) => panic!("Must specify either -r or -R"),
    }.expect("Error processing remotes");

    let mut total_count : usize = 0;
    let mut dup_count : usize = 0;
    for file in WalkBuilder::new(&local).hidden(false).build().filter_map(|x| x.ok()) {
        if file.path().is_dir() == false { total_count += 1; }
        dup_count += dedup_from_set(&file.path(), &checksums) as usize;
    }
    println!("{} files processed. {} Duplicates {}", total_count, dup_count, 
             match args.commit {true => "deleted".to_string(), false => "found".to_string()} );
}
