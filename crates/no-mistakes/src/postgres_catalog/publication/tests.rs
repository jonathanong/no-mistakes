use super::*;

#[test]
fn native_failure_preserves_previous_catalog() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("catalog.json");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/catalog/identifier-names.json");
    std::fs::copy(&fixture, &output).unwrap();
    let mut input = serde_json::json!({
        "connectionEnv": "SAMPLE_DB_URL",
        "schema": "",
        "output": output
    });
    let result = crate::napi_api::generate_postgres_catalog_json_impl(input.clone());
    assert!(result
        .unwrap_err()
        .reason
        .contains("schema must be a non-empty"));
    input["output"] = "".into();
    let error = crate::napi_api::generate_postgres_catalog_json_impl(input).unwrap_err();
    assert!(error.reason.contains("output must be a non-empty"));
    assert_eq!(
        std::fs::read(&output).unwrap(),
        std::fs::read(fixture).unwrap()
    );
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn atomic_publication_preserves_previous_catalog_after_partial_write_failure() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("catalog.json");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/catalog/identifier-names.json");
    std::fs::copy(&fixture, &output).unwrap();
    let original = std::fs::read(&output).unwrap();
    let failure = publish_catalog(&output, |file| {
        // Exercise a real partial temporary-file write before the I/O failure.
        file.write_all(b"partial")?;
        Err(std::io::Error::other("injected write failure"))
    });
    assert!(failure.is_err());
    assert_eq!(std::fs::read(&output).unwrap(), original);
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    assert!(publish_catalog(&directory.path().join("missing/catalog.json"), |_| Ok(())).is_err());
}
