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
