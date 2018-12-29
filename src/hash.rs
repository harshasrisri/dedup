use md5::Md5;
use sha1::Sha1;
use sha2::Sha256;
use sha2::Sha512;
use digest::Digest;
use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::io::Result;
use structopt::StructOpt;
use args::DedupOpts;

const BUFFER_SIZE:usize = 4096;

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

fn hash_algo <D:Digest> (path: &Path) -> Result<String> {
    let mut sh = D::new();
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

pub fn checksum (path: &Path) -> Result<String> {
    let algo = &DedupOpts::from_args().hash_algo.to_string() as &str;
    match algo {
        "MD5"    | "Md5"    | "md5"    => hash_algo::<Md5>    (path),
        "SHA128" | "Sha128" | "sha128" => hash_algo::<Sha1>   (path),
        "SHA256" | "Sha256" | "sha256" => hash_algo::<Sha256> (path),
        "SHA512" | "Sha512" | "sha512" => hash_algo::<Sha512> (path),
        _ => panic!("Unsupported hash algorithm - {}", algo),
    }
}
