use crate::args::CLI_OPTS;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;
use anyhow::Result;

pub trait FileOps : AsRef<Path> {
    fn remove_file(&self) -> Result<()>;
    fn content_equals(&self, other: &Self) -> Result<bool>;
}

impl<P> FileOps for P
where
    P: AsRef<Path> + ?Sized,
{
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
        const CHUNK_SIZE: usize = 4096;
        let mut src = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(self)?;
        let mut tgt = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(other)?;
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
}

// pub fn files_are_equal(src: &PathBuf, tgt: &PathBuf) -> Result<bool> {
// }
