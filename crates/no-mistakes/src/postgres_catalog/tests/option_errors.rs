use super::*;

#[test]
fn rejects_empty_or_nul_search_path_schema_names() {
    for schema in ["", "a\0b"] {
        let error = generate(&PostgresCatalogOptions {
            connection_env: "NOT_SET_NO_MISTAKES_TEST".into(),
            schema: "public".into(),
            coverage: PostgresCatalogCoverage::Complete,
            search_path_schemas: vec![schema.into()],
            current_database: None,
        })
        .unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("searchPathSchemas must contain"),
            "{error}"
        );
    }
}

#[test]
fn rejects_an_empty_connection_environment_variable() {
    const VAR: &str = "NO_MISTAKES_TEST_EMPTY_PG_URL";
    let previous = std::env::var_os(VAR);
    std::env::set_var(VAR, "");
    let error = generate(&PostgresCatalogOptions {
        connection_env: VAR.into(),
        schema: "public".into(),
        coverage: PostgresCatalogCoverage::Complete,
        search_path_schemas: Vec::new(),
        current_database: None,
    })
    .unwrap_err();
    match previous {
        Some(value) => std::env::set_var(VAR, value),
        None => std::env::remove_var(VAR),
    }
    assert!(
        error
            .to_string()
            .starts_with("connection environment variable is empty"),
        "{error}"
    );
}

#[test]
fn generate_postgres_catalog_json_rejects_invalid_options_without_connecting() {
    let invalid_json =
        crate::napi_api::generate_postgres_catalog_json_impl(serde_json::json!("not-an-object"));
    assert!(invalid_json.is_err(), "{invalid_json:?}");

    let empty_schema = crate::napi_api::generate_postgres_catalog_json_impl(serde_json::json!({
        "connectionEnv": "NOT_SET_NO_MISTAKES_TEST",
        "schema": ""
    }));
    let message = empty_schema.unwrap_err().to_string();
    assert!(
        message.contains("schema must be a non-empty PostgreSQL schema name"),
        "{message}"
    );
}
