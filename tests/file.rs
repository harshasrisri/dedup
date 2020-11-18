use dedup::file::FileOps;
use std::path::PathBuf;

#[test]
fn test_equal_file() -> Result<(), Box<dyn std::error::Error>> {
    assert!(PathBuf::from("Cargo.lock").content_equals(&PathBuf::from("Cargo.lock"))?);
    Ok(())
}

#[test]
fn test_unequal_file() -> Result<(), Box<dyn std::error::Error>> {
    assert!(!PathBuf::from("Cargo.lock").content_equals(&PathBuf::from("Cargo.toml"))?);
    Ok(())
}

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

#[test]
fn test_bytes2string() {
    assert_eq!(
        dedup::file::bytes2string(&[1, 2, 3, 4, 5, 6]),
        "010203040506".to_string()
    );
    assert_eq!(
        dedup::file::bytes2string(&[0xca, 0xfe, 0xba, 0xbe]),
        "cafebabe".to_string()
    );
}
