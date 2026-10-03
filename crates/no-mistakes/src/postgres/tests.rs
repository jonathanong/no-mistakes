use super::*;

#[test]
fn atomic_publication_preserves_previous_catalog_after_partial_write_failure() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("catalog.json");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/catalog/identifier-names.json");
    std::fs::copy(&fixture, &output).unwrap();
    let original = std::fs::read(&output).unwrap();
    let failure = publish_catalog(&output, &mut |file| {
        // Exercise a real partial temporary-file write before the I/O failure.
        file.write_all(b"partial")?;
        Err(std::io::Error::other("injected write failure"))
    });
    assert!(failure.is_err());
    assert_eq!(std::fs::read(&output).unwrap(), original);
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    assert!(
        publish_catalog(&directory.path().join("missing/catalog.json"), &mut |_| Ok(
            ()
        ))
        .is_err()
    );
}
