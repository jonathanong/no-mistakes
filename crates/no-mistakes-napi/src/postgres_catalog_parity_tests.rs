use no_mistakes::postgres_catalog::{generate, PostgresCatalogCoverage, PostgresCatalogOptions};
use std::path::Path;

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
    let result = crate::generate_postgres_catalog_json_impl(input.clone());
    assert!(result
        .unwrap_err()
        .reason
        .contains("schema must be a non-empty"));
    input["output"] = "".into();
    let error = crate::generate_postgres_catalog_json_impl(input).unwrap_err();
    assert!(error.reason.contains("output must be a non-empty"));
    assert_eq!(
        std::fs::read(&output).unwrap(),
        std::fs::read(fixture).unwrap()
    );
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn native_publication_rejects_a_non_path_output() {
    let error = crate::generate_postgres_catalog_json_impl(serde_json::json!({
        "connectionEnv": "SAMPLE_DB_URL",
        "output": false
    }))
    .unwrap_err();
    assert!(error.reason.contains("expected path string"));
}

#[test]
fn generate_postgres_catalog_json_rejects_invalid_options_without_connecting() {
    let invalid_json =
        crate::generate_postgres_catalog_json_impl(serde_json::json!("not-an-object"));
    assert!(invalid_json.is_err(), "{invalid_json:?}");

    let empty_schema = crate::generate_postgres_catalog_json_impl(serde_json::json!({
        "connectionEnv": "NOT_SET_NO_MISTAKES_TEST",
        "schema": ""
    }));
    let message = empty_schema.unwrap_err().to_string();
    assert!(
        message.contains("schema must be a non-empty PostgreSQL schema name"),
        "{message}"
    );
}

#[test]
fn generate_postgres_catalog_json_matches_library_on_real_postgres() {
    let Some(connection) = std::env::var_os("NO_MISTAKES_TEST_POSTGRES_URL") else {
        assert!(
            std::env::var_os("CI").is_none(),
            "CI must provide NO_MISTAKES_TEST_POSTGRES_URL for real PostgreSQL catalog tests"
        );
        return;
    };
    let _ = connection;
    let options = PostgresCatalogOptions {
        connection_env: "NO_MISTAKES_TEST_POSTGRES_URL".into(),
        schema: "public".into(),
        coverage: PostgresCatalogCoverage::Complete,
        search_path_schemas: Vec::new(),
        current_database: None,
    };
    let catalog = generate(&options).unwrap();
    let napi = crate::generate_postgres_catalog_json_impl(serde_json::to_value(&options).unwrap())
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&napi).unwrap(),
        catalog
    );
    let mut options_json = serde_json::to_value(&options).unwrap();
    options_json["currentDatabase"] = "app".into();
    let recorded = crate::generate_postgres_catalog_json_impl(options_json).unwrap();
    let mut expected = catalog.clone();
    expected["currentDatabase"] = "app".into();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&recorded).unwrap(),
        expected
    );

    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("catalog.json");
    let mut input = serde_json::to_value(&options).unwrap();
    input["output"] = serde_json::to_value(&output).unwrap();
    let returned = crate::generate_postgres_catalog_json_impl(input.clone()).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&returned).unwrap(),
        catalog
    );
    let published = std::fs::read_to_string(&output).unwrap();
    assert_eq!(published, format!("{catalog:#}\n"));
    input["schema"] = "catalog_fixture_missing".into();
    assert!(crate::generate_postgres_catalog_json_impl(input).is_err());
    assert_eq!(std::fs::read_to_string(&output).unwrap(), published);
}
