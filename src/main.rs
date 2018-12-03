extern crate md5;

use std::fs;
use std::env;
use std::io::Result;
use std::fs::File;
use std::path::Path;
use std::io::{BufRead, BufReader, Read};
use std::collections::HashSet;
use md5::{Md5,Digest};

const BUFFER_SIZE:usize = 1024;

#[test]
fn test_bytes2string () {
    assert_eq!(bytes2string(&[1,2,3,4,5,6]).unwrap(), "010203040506".to_string());
    assert_eq!(bytes2string(&[0xca, 0xfe, 0xba, 0xbe]).unwrap(), "cafebabe".to_string());
}

fn bytes2string (byte_array: &[u8]) -> Result<String> {
    let mut ret = String::from("");
    for byte in byte_array {
        ret.push_str(&format!("{:02x}", byte));
    }
    Ok(ret)
}

fn md5 (path: &Path) -> Result<String> {
    let mut sh = Md5::default();
    let mut file = File::open(&path)?;
    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        let n = file.read(&mut buffer)?;
        sh.input(&buffer[..n]);
        if n == 0 || n < BUFFER_SIZE {
            break;
        }
    }
    bytes2string(&sh.result())
}

fn dedup_from_set (dir : &Path, checksums : &mut HashSet<String>) -> Result<()> {
    if dir.is_dir() == false {
        return Ok(());
    }

    println!("Processing directory {}", &dir.to_str().unwrap());

    for path in fs::read_dir(dir)? {
        let path = path.unwrap();
        if path.path().is_dir() == true {
            dedup_from_set (&path.path(), checksums);
        } else {
            let chksum = match md5(&path.path()) {
                Ok(sum) => sum,
                Err(_)  => continue
            };
            if checksums.contains(&chksum) {
                println!("rm -v {}", &path.path().to_str().unwrap());
            }
        }
    }
    Ok(())
}

fn main() {
    let args : Vec<String> = std::env::args().collect();

    assert!(args.len() == 3);

    let remote = File::open(&args[1]).expect("File not found");
    let local = Path::new(&args[2]);

    let mut checksums = HashSet::new();

    for line in BufReader::new(remote).lines().filter_map(|result| result.ok()) {
        let hashpath : Vec<&str> = line.splitn(2, ' ').collect();
        checksums.insert(hashpath[0].to_string());
    }

    dedup_from_set(&local, &mut checksums);
}
