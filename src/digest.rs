use anyhow::Result;
use std::{fmt::Display, path::Path, str::FromStr};
use std::fs::OpenOptions;
use std::io::Read;

#[derive(Debug, Clone)]
pub enum DigestKind {
    MD5,
    SHA1,
    SHA2,
}

impl FromStr for DigestKind {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let digest = s.to_ascii_lowercase();
        match digest.as_str() {
            "md5" => Ok(Self::MD5),
            "sha1" | "sha128" => Ok(Self::SHA1),
            "sha2" | "sha256" => Ok(Self::SHA2),
            _ => Err(format!("Unsupported/Invalid digest algorithm: {digest}")),
        }
    }
}

impl Display for DigestKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::MD5 => "MD5",
                Self::SHA1 => "SHA1",
                Self::SHA2 => "SHA2",
            }
        )
    }
}

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
    fn digest(&self, digest: DigestKind) -> Result<String>;
}

impl<P> DigestFile for P
where 
    P: AsRef<Path>,
{
    fn digest(&self, digest: DigestKind) -> Result<String> {
        match digest {
            DigestKind::MD5 => content_digest::<P, md5::Md5>(self),
            DigestKind::SHA1 => content_digest::<P, sha1::Sha1>(self),
            DigestKind::SHA2 => content_digest::<P, sha2::Sha256>(self),
        }
    }
}

fn content_digest<P, D>(path: &P) -> Result<String>
where
    P: AsRef<Path> + ?Sized,
    D: digest::Digest,
{
    let inner = OpenOptions::new().read(true).write(false).create(false).open(path)?;
    let chunk_iter = BufChunkIterator { inner, chunk_size: CHUNK_SIZE };
    let mut sh = D::new();
    for chunk in chunk_iter {
        sh.input(chunk);
    }
    Ok(hex::encode(sh.result()))
}
