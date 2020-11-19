use crate::args::CLI_OPTS;
use anyhow::Result;
use digest::Digest;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::Path;

const CHUNK_SIZE: usize = 4096;

pub fn bytes2string(byte_array: &[u8]) -> String {
    byte_array
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

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

pub trait FileOps: AsRef<Path> {
    fn remove_file(&self) -> Result<()>;
    fn content_equals(&self, other: &Self) -> Result<bool>;
    fn content_checksum<D: Digest>(&self) -> Result<String>;
    fn open_ro(&self) -> Result<File>;
    fn chunks(&self, chunk_size: usize) -> Result<BufChunkIterator<File>>;
}

impl<P> FileOps for P
where
    P: AsRef<Path> + ?Sized,
{
    fn open_ro(&self) -> Result<File> {
        Ok(OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(self)?)
    }

    fn remove_file(&self) -> Result<()> {
        if CLI_OPTS.verbose > 0 {
            println!("{}", &self.as_ref().to_str().unwrap());
        }

        if CLI_OPTS.commit {
            std::fs::remove_file(self)?;
        }

        Ok(())
    }

    fn content_equals(&self, other: &Self) -> Result<bool> {
        Ok(self
            .chunks(CHUNK_SIZE)?
            .into_iter()
            .zip(other.chunks(CHUNK_SIZE)?.into_iter())
            .all(|(c1, c2)| c1 == c2))
    }

    fn content_checksum<D: Digest>(&self) -> Result<String> {
        let mut sh = D::new();
        self.chunks(CHUNK_SIZE)?
            .into_iter()
            .for_each(|chunk| sh.input(chunk));
        Ok(bytes2string(&sh.result()))
    }

    fn chunks(&self, chunk_size: usize) -> Result<BufChunkIterator<File>> {
        Ok(BufChunkIterator {
            inner: self.open_ro()?,
            chunk_size,
        })
    }
}
