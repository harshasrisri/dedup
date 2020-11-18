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

// pub struct BufChunkIterator<R: Read> {
//     inner: BufReader<R>,
// }

// impl<R: Read> Iterator for BufChunkIterator<R> {
//     type Item = Vec<u8>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let ret = self.inner.fill_buf().map(|slice| slice.to_owned()).ok();
//         println!("{:#?}", ret);
//         ret.iter().for_each(|buf| self.inner.consume(buf.len()));
//         ret
//     }
// }

pub trait FileOps: AsRef<Path> {
    fn remove_file(&self) -> Result<()>;
    fn content_equals(&self, other: &Self) -> Result<bool>;
    fn content_checksum<D: Digest>(&self) -> Result<String>;
    fn open_ro(&self) -> Result<File>;
    // fn chunks(&self, chunk_size: usize) -> Result<BufChunkIterator<File>>;
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

    // fn content_equals(&self, other: &Self) -> Result<bool> {
    //     Ok(
    //         self.chunks(CHUNK_SIZE)?.into_iter()
    //         .zip(other.chunks(CHUNK_SIZE)?.into_iter())
    //         .all(|(c1, c2)| c1 == c2)
    //       )
    // }

    // fn content_checksum<D: Digest>(&self) -> Result<String> {
    //     let mut sh = D::new();
    //     self.chunks(CHUNK_SIZE)?.into_iter().for_each(|chunk| sh.input(chunk));
    //     Ok(bytes2string(&sh.result()))
    // }

    fn content_equals(&self, other: &Self) -> Result<bool> {
        let mut src = self.open_ro()?;
        let mut tgt = other.open_ro()?;
        let mut src_buf = Vec::with_capacity(CHUNK_SIZE);
        let mut tgt_buf = Vec::with_capacity(CHUNK_SIZE);

        loop {
            src_buf.clear();
            tgt_buf.clear();
            let src_len = src
                .by_ref()
                .take(CHUNK_SIZE as u64)
                .read_to_end(&mut src_buf)?;
            let tgt_len = tgt
                .by_ref()
                .take(CHUNK_SIZE as u64)
                .read_to_end(&mut tgt_buf)?;
            if src_len == 0 && tgt_len == 0 {
                return Ok(true);
            } else if src_len != tgt_len || src_buf != tgt_buf {
                return Ok(false);
            }
        }
    }

    fn content_checksum<D: Digest>(&self) -> Result<String> {
        let mut sh = D::new();
        let mut file = self.open_ro()?;
        let mut buffer = [0u8; CHUNK_SIZE];
        loop {
            let n = file.read(&mut buffer)?;
            sh.input(&buffer[..n]);
            if n == 0 || n < CHUNK_SIZE {
                break;
            }
        }
        Ok(bytes2string(&sh.result()))
    }

    // fn chunks(&self, chunk_size: usize) -> Result<BufChunkIterator<File>> {
    //     Ok(
    //         BufChunkIterator {
    //             inner: BufReader::with_capacity(chunk_size, self.open_ro()?)
    //         }
    //       )
    // }
}
