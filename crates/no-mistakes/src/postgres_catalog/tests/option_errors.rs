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

struct RestoreEnvVar {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl RestoreEnvVar {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var_os(key);
        // The name is unique to this test. Restore on drop so a panic in
        // `generate` cannot leave the empty value in the process.
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for RestoreEnvVar {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

#[test]
fn rejects_an_empty_connection_environment_variable() {
    const VAR: &str = "NO_MISTAKES_TEST_EMPTY_PG_URL";
    let _restore = RestoreEnvVar::set(VAR, "");
    let error = generate(&PostgresCatalogOptions {
        connection_env: VAR.into(),
        schema: "public".into(),
        coverage: PostgresCatalogCoverage::Complete,
        search_path_schemas: Vec::new(),
        current_database: None,
    })
    .unwrap_err();
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
