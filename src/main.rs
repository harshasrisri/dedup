#[macro_use]
extern crate structopt;
extern crate md5;
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
        println!("Removing file {}", &filepath.to_str().unwrap());
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

fn main () {
    let args = DedupOpts::from_args();
    println!("{:?}", args);

    let local = &args.local_path;
    if local.exists() == false {
        println!("Local path not found - {}", local.to_str().unwrap());
        return
    }

    let remote = match File::open(&args.remote_list.to_str().unwrap()) {
        Ok(file_handle) => file_handle,
        Err(_) => {
            println!("Remote list file not found - {}", &&args.remote_list.to_str().unwrap());
            return
        }
    };

    let mut checksums = HashSet::new();
    for line in BufReader::new(remote).lines().filter_map(|result| result.ok()) {
        let hashpath : Vec<&str> = line.splitn(2, ' ').collect();
        checksums.insert(hashpath[0].to_string());
    }

    let mut total_count : usize = 0;
    let mut dup_count : usize = 0;
    for filepath in WalkBuilder::new(&local).hidden(false).build() {
        total_count += 1;
        match filepath {
            Ok(file) => dup_count += dedup_from_set(&file.path(), &checksums) as usize,
            Err(err) => println!("Error encountered {}", err),
        }
    }
    println!("{} files processed. {} Duplicates found", total_count, dup_count);
}
