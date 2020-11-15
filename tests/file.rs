use std::path::PathBuf;

#[test]
fn test_equal_file() -> Result<(), Box<dyn std::error::Error>> {
    assert!(dedup::file::files_are_equal(
        &PathBuf::from("Cargo.toml"),
        &PathBuf::from("Cargo.toml")
    )?);
    Ok(())
}

#[test]
fn test_unequal_file() -> Result<(), Box<dyn std::error::Error>> {
    assert!(!dedup::file::files_are_equal(
        &PathBuf::from("Cargo.lock"),
        &PathBuf::from("Cargo.toml")
    )?);
    Ok(())
}

#[test]
fn test_bytes2string() {
    assert_eq!(
        dedup::hash::bytes2string(&[1, 2, 3, 4, 5, 6]).unwrap(),
        "010203040506".to_string()
    );
    assert_eq!(
        dedup::hash::bytes2string(&[0xca, 0xfe, 0xba, 0xbe]).unwrap(),
        "cafebabe".to_string()
    );
}
