use std::{fmt::Display, str::FromStr};

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
            "md5" => Ok(DigestKind::MD5),
            "sha1" | "sha128" => Ok(DigestKind::SHA1),
            "sha2" | "sha256" => Ok(DigestKind::SHA2),
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
                DigestKind::MD5 => "MD5",
                DigestKind::SHA1 => "SHA1",
                DigestKind::SHA2 => "SHA2",
            }
        )
    }
}
