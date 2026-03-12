use anyhow::Result;
use twox_hash::XxHash3_64;
use std::hash::Hasher;
use std::path::Path;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};

pub trait HashFile: AsRef<Path> {
    fn chksum(&self) -> Result<(usize, String)>;
}

impl<P> HashFile for P
where 
    P: AsRef<Path>,
{
    fn chksum(&self) -> Result<(usize, String)> {
        let mut sh = XxHash3_64::with_seed(0xdeadbeef);
        let capacity = 256 * 1024; // 256 KB
        let inner = OpenOptions::new().read(true).write(false).create(false).open(self)?;
        let mut br = BufReader::with_capacity(capacity, inner);
        let mut file_size = 0;
        loop {
            let buf = br.fill_buf()?;
            if buf.is_empty() { break; }
            let buflen = buf.len();
            file_size += buflen;
            sh.write(buf);
            br.consume(buflen);
        }

        Ok((file_size, format!("{:X}", sh.finish())))
    }
}
