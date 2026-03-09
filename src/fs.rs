use anyhow::Result;
use log::{error, trace};
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, BufReader};
use walkdir::WalkDir;

const CHUNK_SIZE: usize = 1024 * 1024;

#[allow(async_fn_in_trait)]
pub trait FileOps: AsRef<Path> {
    async fn remove_file(&self, commit: bool) -> Result<()>;
    async fn open_ro(&self) -> Result<File>;
    async fn open_rw(&self) -> Result<File>;
    async fn dup_of(&self, other: &Self) -> Result<bool>;
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

    async fn open_rw(&self) -> Result<File> {
        trace!("{}: opening file in RW mode", self.as_ref().display());
        Ok(OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(self)
            .await?)
    }

    async fn dup_of(&self, other: &Self) -> Result<bool> {
        let (this, that) = (self.open_ro().await?, other.open_ro().await?);
        let mut this = BufReader::with_capacity(CHUNK_SIZE, this);
        let mut that = BufReader::with_capacity(CHUNK_SIZE, that);

        loop {
            let this_slice = this.fill_buf().await?;
            let that_slice = that.fill_buf().await?;

            if this_slice != that_slice {
                return Ok(false);
            }

            if this_slice.is_empty() && that_slice.is_empty() {
                return Ok(true);
            }

            let (this_len, that_len) = (this_slice.len(), that_slice.len());

            this.consume(this_len);
            that.consume(that_len);
        }
    }
}

pub trait DirOps {
    fn walkdir(&self) -> impl Iterator<Item = PathBuf>;
}

impl<P> DirOps for P
where
    P: AsRef<Path> + ?Sized,
{
    fn walkdir(&self) -> impl Iterator<Item = PathBuf> {
        WalkDir::new(self).into_iter().filter_map(|entry| {
            let path = entry
                .inspect_err(|e| error!("{}: error while walking: {e}", self.as_ref().display()))
                .ok()?
                .into_path();
            (!path.is_symlink() && !path.is_dir()).then_some(path)
        })
    }
}
