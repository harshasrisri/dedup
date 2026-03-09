use anyhow::Result;
use twox_hash::XxHash64;
use std::hash::Hasher;
use std::path::Path;
use std::fs::OpenOptions;
use std::io::Read;

const CHUNK_SIZE: usize = 4096;

pub struct BufChunkIterator<R> {
    inner: R,
    chunk_size: usize,
}

impl<R: Read> Iterator for BufChunkIterator<R> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = Vec::with_capacity(self.chunk_size);
        let len = self
            .inner
            .by_ref()
            .take(self.chunk_size as u64)
            .read_to_end(&mut buffer)
            .ok();
        if len != Some(0) {
            len.map(|len| buffer.truncate(len));
            Some(buffer)
        } else {
            None
        }
    }
}

pub trait DigestFile: AsRef<Path> {
    fn chksum(&self) -> Result<String>;
}

impl<P> DigestFile for P
where 
    P: AsRef<Path>,
{
    fn chksum(&self) -> Result<String> {
        let mut sh = XxHash64::with_seed(0xdeadbeef);
        let file = OpenOptions::new().read(true).write(false).create(false).open(self)?;
        let mut reader = BufReader::with_capacity(CHUNK_SIZE, file);
        let mut buffer = Vec::with_capacity(CHUNK_SIZE);

        while let Ok(size) = reader.read(&mut buffer) {
            if size == 0 {
                break;
            }
            sh.write(&buffer);
        }

        Ok(format!("{:X}", sh.finish()))
    }
}
