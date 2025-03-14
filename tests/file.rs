use dedup::file::FileOps;
use std::path::PathBuf;

#[test]
fn test_equal_file_checksum() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        PathBuf::from("Cargo.lock").content_checksum::<md5::Md5>()?,
        PathBuf::from("Cargo.lock").content_checksum::<md5::Md5>()?
    );
    Ok(())
}

#[test]
fn test_unequal_file_checksum() -> Result<(), Box<dyn std::error::Error>> {
    assert_ne!(
        PathBuf::from("Cargo.lock").content_checksum::<md5::Md5>()?,
        PathBuf::from("Cargo.toml").content_checksum::<md5::Md5>()?
    );
    Ok(())
}
