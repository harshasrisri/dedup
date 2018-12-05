extern crate md5;
extern crate ignore;

use std::io::Result;
use std::fs::File;
use std::path::Path;
use std::io::{BufRead, BufReader, Read};
use std::collections::HashSet;
use md5::{Md5,Digest};
use ignore::Walk;

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

fn dedup_from_set (filepath : &Path, checksums : &HashSet<String>) {
    if filepath.is_dir() == true {
        return;
    }

    let chksum = md5(&filepath).expect(&format!("Error calculating MD5Sum of {}", &filepath.to_str().unwrap()));

    if checksums.contains(&chksum) {
        println!("rm -v {}", &filepath.to_str().unwrap());
    }
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

    for filepath in Walk::new(&local) {
        match filepath {
            Ok(file) => dedup_from_set(&file.path(), &checksums),
            Err(err) => println!("Error encountered {}", err),
        }
    }
}
