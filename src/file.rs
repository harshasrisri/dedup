use anyhow::Result;
use digest::Digest;
use log::{debug, trace};
use std::hash::Hasher;
use std::path::Path;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use twox_hash::XxHash64;

const CHUNK_SIZE: usize = 4096;

pub trait FileOps: AsRef<Path> {
    fn remove_file(&self, commit: bool) -> impl Future<Output = Result<()>>;
    fn content_digest<D: Digest>(&self) -> impl Future<Output = Result<String>>;
    fn content_chksum(&self) -> impl Future<Output = Result<String>>;
    fn open_ro(&self) -> impl Future<Output = Result<File>>;
}

impl<P> FileOps for P
where
    P: AsRef<Path> + ?Sized,
{
    async fn open_ro(&self) -> Result<File> {
        trace!("{}: opening file in RO mode", self.as_ref().display());
        Ok(OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(self)
            .await?)
    }

    async fn remove_file(&self, commit: bool) -> Result<()> {
        if commit {
            trace!("{}: removing file", self.as_ref().display());
            tokio::fs::remove_file(self).await?;
        } else {
            trace!("{}: candidate for removal", self.as_ref().display());
        }
        Ok(())
    }

    async fn content_digest<D: Digest>(&self) -> Result<String> {
        let mut sh = D::new();
        let file = self.open_ro().await?;
        let mut reader = BufReader::with_capacity(CHUNK_SIZE, file);
        let mut file_size = 0;

        while let Ok(slice) = reader.fill_buf().await {
            let len = slice.len();
            if len != 0 {
                sh.input(slice);
                trace!("{}: ingesting {len} bytes", self.as_ref().display());
                file_size += len;
                let _ = slice;
            } else {
                break;
            }
            reader.consume(len);
        }
        trace!("{}: digested {file_size} bytes", self.as_ref().display());
        Ok(hex::encode(sh.result()))
    }

    async fn content_chksum(&self) -> Result<String> {
        let mut sh = XxHash64::with_seed(0xdeadbeef);
        let file = self.open_ro().await?;
        let mut reader = BufReader::with_capacity(CHUNK_SIZE, file);
        let mut buffer = Vec::with_capacity(CHUNK_SIZE);

        while let Ok(size) = reader.read(&mut buffer).await {
            if size == 0 {
                break;
            }
            sh.write(&buffer);
        }
        Ok(format!("{:X}", sh.finish()))
    }
}
