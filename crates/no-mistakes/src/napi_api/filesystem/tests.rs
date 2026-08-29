use super::rename_no_replace_impl;

#[test]
fn does_not_replace_an_existing_destination() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("source");
    let destination = directory.path().join("destination");
    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(&destination).unwrap();

    assert!(!rename_no_replace_impl(&source, &destination).unwrap());
    assert!(source.is_dir());
    assert!(destination.is_dir());

    std::fs::remove_dir(&destination).unwrap();
    assert!(rename_no_replace_impl(&source, &destination).unwrap());
    assert!(!source.exists());
    assert!(destination.is_dir());
}
