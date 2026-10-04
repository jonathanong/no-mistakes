use super::*;
use crate::codebase::postgres::{CatalogCoverage, ResolvedArbiter, SchemaCatalog};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::sync::Arc;

#[test]
fn rejects_invalid_options_without_accessing_connection_secrets() {
    for (connection_env, schema) in [
        ("", "public"),
        ("a=b", "public"),
        ("a\0b", "public"),
        ("NOT_SET_NO_MISTAKES_TEST", "a\0b"),
        ("NOT_SET_NO_MISTAKES_TEST", ""),
        ("NOT_SET_NO_MISTAKES_TEST", "public"),
    ] {
        assert!(generate(&PostgresCatalogOptions {
            connection_env: connection_env.into(),
            schema: schema.into(),
            coverage: PostgresCatalogCoverage::Complete,
            search_path_schemas: Vec::new(),
        })
        .is_err());
    }
}

#[test]
fn observed_catalog_preserves_postgres_ordering_state() {
    if std::env::var_os("NO_MISTAKES_TEST_POSTGRES_URL").is_none() {
        assert!(
            std::env::var_os("CI").is_none(),
            "CI must provide NO_MISTAKES_TEST_POSTGRES_URL for real PostgreSQL catalog tests"
        );
        return;
    }
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/catalog-generation");
    let connection = std::env::var("NO_MISTAKES_TEST_POSTGRES_URL").unwrap();
    for (file, success) in [
        ("setup.sql", true),
        ("search-path-privileges.sql", true),
        ("invalid-index.sql", false),
        ("deferrable-conflict.sql", false),
    ] {
        let output = {
            let mut command = Command::new("psql");
            connection_environment(&connection, &mut command).unwrap();
            command
        }
        .args(["-X", "--no-password", "-v", "ON_ERROR_STOP=1", "-f"])
        .arg(fixture.join(file))
        .output()
        .unwrap();
        assert_eq!(
            output.status.success(),
            success,
            "fixture execution status for {file}"
        );
    }
    let options = PostgresCatalogOptions {
        connection_env: "NO_MISTAKES_TEST_POSTGRES_URL".into(),
        schema: "Catalog.Test".into(),
        coverage: PostgresCatalogCoverage::Ordering,
        search_path_schemas: Vec::new(),
    };
    let catalog = generate(&options).unwrap();
    assert_eq!(
        catalog,
        generate(&options).unwrap(),
        "unchanged committed schema generates identical JSON"
    );
    assert_eq!(catalog["coverage"], "ordering");
    let observed_database = {
        let mut command = Command::new("psql");
        connection_environment(&connection, &mut command).unwrap();
        command
            .args([
                "-X",
                "--no-password",
                "-At",
                "-c",
                "SELECT current_database()",
            ])
            .output()
            .unwrap()
    };
    assert!(observed_database.status.success());
    let observed_database = String::from_utf8(observed_database.stdout).unwrap();
    assert_eq!(catalog["currentDatabase"], observed_database.trim());

    assert!(catalog.get("searchPathEvidence").is_none());
    let with_evidence = generate(&PostgresCatalogOptions {
        search_path_schemas: vec![
            "pg_catalog".into(),
            "Catalog.Test".into(),
            "Catalog.Locked".into(),
            "no_mistakes_missing_search_path_schema".into(),
        ],
        ..options.clone()
    })
    .unwrap();
    assert!(with_evidence["searchPathEvidence"]["Catalog.Test"]
        .as_array()
        .is_some_and(|names| names.iter().any(|name| name == "Items")));
    assert!(with_evidence["searchPathEvidence"]["pg_catalog"]
        .as_array()
        .is_some_and(|names| names.iter().any(|name| name == "pg_class")));
    assert!(with_evidence["searchPathEvidence"]["Catalog.Locked"]
        .as_array()
        .is_some_and(|names| names.iter().any(|name| name == "accounts")));
    assert!(
        with_evidence["searchPathEvidence"]["no_mistakes_missing_search_path_schema"].is_null()
    );
    let restricted_query = format!(
        "SET ROLE no_mistakes_catalog_no_usage;\n{}",
        sql::catalog_query_with_search_path(
            "Catalog.Test",
            PostgresCatalogCoverage::Ordering,
            &["Catalog.Locked".into()],
        )
    );
    let mut restricted = Command::new("psql");
    connection_environment(&connection, &mut restricted).unwrap();
    let output = restricted
        .args([
            "-X",
            "--quiet",
            "--tuples-only",
            "--no-align",
            "--no-password",
            "-v",
            "ON_ERROR_STOP=1",
            "-c",
            &restricted_query,
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "restricted catalog query failed");
    let restricted: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(restricted["searchPathEvidence"]["Catalog.Locked"].is_null());
    let escaped = generate(&PostgresCatalogOptions {
        schema: "Catalog\\'Schema".into(),
        ..options.clone()
    })
    .unwrap();
    assert_eq!(escaped["schema"], "Catalog\\'Schema");
    assert!(escaped["tables"].get("safe").is_some());

    let napi = crate::napi_api::generate_postgres_catalog_json_impl(
        serde_json::to_value(&options).unwrap(),
    )
    .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&napi).unwrap(),
        catalog
    );

    assert_eq!(
        catalog["tables"]["\"Items\""]["indexes"]["expression_key"]["valid"],
        true
    );
    assert_eq!(
        catalog["tables"]["\"Items\""]["indexes"]["expression_key"]["ready"],
        true
    );
    let table = &catalog["tables"]["\"Items\""];
    assert_eq!(table["columns"]["score"]["generated"], "stored");
    assert_eq!(
        table["indexes"]["partial_desc"]["keys"][0]["descending"],
        true
    );
    assert_eq!(
        table["indexes"]["partial_desc"]["keys"][0]["nullsFirst"],
        false
    );
    assert_eq!(
        table["indexes"]["partial_desc"]["keys"][1]["nullsFirst"],
        true
    );
    assert!(table["indexes"]["partial_desc"]["predicate"]
        .as_str()
        .unwrap()
        .contains("active"));
    assert!(table["indexes"]["expression_key"]["keys"][0]["expression"]
        .as_str()
        .unwrap()
        .contains("lower(email)"));
    assert_eq!(
        table["indexes"]["included_key"]["keys"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        catalog["tables"]["duplicates"]["indexes"]["invalid_duplicate"]["valid"],
        false
    );
    assert_eq!(
        catalog["tables"]["duplicates"]["indexes"]["invalid_duplicate"]["ready"],
        false
    );
    // Load exactly the generated public contract through the normal catalog reader.
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("catalog.json");
    std::fs::write(&path, serde_json::to_vec(&catalog).unwrap()).unwrap();
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[path])));
    let loaded = SchemaCatalog::load(directory.path(), "catalog.json", &sources).unwrap();
    assert_eq!(loaded.coverage(), CatalogCoverage::Ordering);
    // Feed the observed catalog through both configured guards, not just the resolver.
    let config = |rule: &str| {
        let mut config = crate::config::v2::NoMistakesConfig::default();
        config.rules.push(crate::config::v2::schema::RuleDef {
            rule: rule.into(),
            scope: Some(crate::config::v2::schema::RuleScope::Repository),
            options: serde_yaml::from_str("schemaCatalogPath: catalog.json\nimportSpecifier: fixture-db\nexecutorNames: [query]").unwrap(),
            ..Default::default()
        });
        config
    };
    for (name, expected) in [
        ("conflict-pass.ts", 0),
        ("conflict-fail.ts", 1),
        ("lock-pass.ts", 0),
        ("lock-fail.ts", 1),
    ] {
        let source = directory.path().join(name);
        std::fs::copy(fixture.join(name), &source).unwrap();
        let files = [source, directory.path().join("catalog.json")];
        let findings = if name.starts_with("conflict") {
            crate::codebase::rules::postgres_conflict_ordering::check_with_files(
                directory.path(),
                &config("postgres-conflict-ordering"),
                &files,
            )
            .unwrap()
        } else {
            crate::codebase::rules::postgres_lock_ordering::check_with_files(
                directory.path(),
                &config("postgres-lock-ordering"),
                &files,
            )
            .unwrap()
        };
        assert_eq!(
            findings.len(),
            expected,
            "observed catalog rule outcome for {name}: {findings:#?}"
        );
    }

    assert_eq!(
        catalog["tables"]["deferred"]["indexes"]["deferred_key"]["immediate"],
        false
    );
    assert_eq!(
        loaded.resolve_columns("deferred", &["value".into()], None),
        ResolvedArbiter::Unresolved
    );
    assert_eq!(
        loaded.resolve_constraint("deferred", "deferred_key"),
        ResolvedArbiter::Unresolved
    );
    assert!(loaded.has_canonical_prefix(
        "deferred",
        &[crate::codebase::postgres::CanonicalOrderKey {
            expression: "value".into(),
            ascending: true,
            nulls_first: false
        }]
    ));
    for name in ["special_opclass", "special_collation"] {
        assert_eq!(
            catalog["tables"]["special"]["indexes"][name]["keys"][0]["orderingSupported"],
            false
        );
    }
    assert_eq!(
        loaded.resolve_columns("special", &["value".into()], None),
        ResolvedArbiter::Unresolved
    );
    assert!(!loaded.has_canonical_prefix(
        "special",
        &[crate::codebase::postgres::CanonicalOrderKey {
            expression: "value".into(),
            ascending: true,
            nulls_first: false
        }]
    ));

    assert_eq!(
        catalog["tables"]["mixed_collation"]["indexes"]["mixed_default"]["keys"][0]
            ["orderingSupported"],
        false
    );
    for table in ["mixed_collation", "mixed_deferred"] {
        assert_eq!(
            loaded.resolve_columns(table, &["value".into()], None),
            ResolvedArbiter::Unresolved
        );
        assert!(loaded.has_canonical_prefix(
            table,
            &[crate::codebase::postgres::CanonicalOrderKey {
                expression: "value".into(),
                ascending: true,
                nulls_first: false,
            }]
        ));
    }
    assert!(matches!(
        loaded.resolve_columns("numeric_expression", &["value + 1".into()], None),
        ResolvedArbiter::Exact(_)
    ));
    assert_eq!(
        catalog["tables"]["\"Items\""]["indexes"]["expression_key"]["keys"][0]["orderingSupported"],
        false
    );

    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    assert!(facts.postgres_ordering_catalog("../catalog.json").is_err());
    facts
        .postgres_schema_catalogs
        .insert("catalog.json".into(), Ok(Arc::new(loaded.clone())));
    assert!(facts.postgres_ordering_catalog("catalog.json").is_ok());
    assert!(facts
        .postgres_schema_catalog("catalog.json")
        .unwrap_err()
        .to_string()
        .contains("requires a complete schema catalog"));

    assert!(matches!(
        loaded.resolve_constraint("\"Catalog.Test\".\"Items\"", "\"Email.Unique\""),
        ResolvedArbiter::Exact(_)
    ));
    assert_eq!(
        loaded.resolve_columns("duplicates", &["value".into()], None),
        ResolvedArbiter::Unresolved
    );
    assert_eq!(
        loaded.resolve_columns("other.duplicates", &["value".into()], None),
        ResolvedArbiter::Unresolved
    );
    assert!(generate(&PostgresCatalogOptions {
        schema: "missing' schema".into(),
        ..options
    })
    .is_err());
}

#[test]
fn url_credentials_stay_in_child_environment() {
    let mut command = Command::new("psql");
    connection_environment(
        "postgresql://user%2Bname:secret%40value@[::1]:5433/database%20name?sslmode=require",
        &mut command,
    )
    .unwrap();
    assert_eq!(command.get_args().count(), 0);
    let environment = command
        .get_envs()
        .filter_map(|(key, value)| {
            value.map(|value| {
                (
                    key.to_string_lossy().into_owned(),
                    value.to_string_lossy().into_owned(),
                )
            })
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(environment["PGHOST"], "::1");
    assert_eq!(environment["PGUSER"], "user+name");
    assert_eq!(environment["PGPASSWORD"], "secret@value");
    assert_eq!(environment["PGDATABASE"], "database name");
    assert_eq!(environment["PGSSLMODE"], "require");
    assert_eq!(environment["PGCONNECT_TIMEOUT"], "10");
    for invalid in [
        "host=localhost dbname=postgres",
        "https://localhost/db",
        "postgresql://localhost/db#fragment",
        "postgresql://localhost/db?unknown=secret",
        "postgresql://user%FF@localhost/db",
        "postgresql://user:password%FF@localhost/db",
        "postgresql://localhost/db%FF",
    ] {
        assert!(connection_environment(invalid, &mut Command::new("psql")).is_err());
    }
}

#[test]
fn supported_url_parameters_map_to_libpq_environment() {
    let mut command = Command::new("psql");
    connection_environment("postgresql://localhost/db?host=localhost&port=5432&user=reader&dbname=test&sslmode=verify-full&sslrootcert=root.pem&sslcert=client.pem&sslkey=client.key&connect_timeout=5", &mut command).unwrap();
    let environment = command
        .get_envs()
        .filter(|(_, value)| value.is_some())
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(environment.len(), 9);
    assert_eq!(
        environment[std::ffi::OsStr::new("PGCONNECT_TIMEOUT")],
        Some(std::ffi::OsStr::new("5"))
    );
}

#[test]
fn url_can_use_libpq_defaults_for_omitted_components() {
    let mut command = Command::new("psql");
    connection_environment("postgresql:///", &mut command).unwrap();
    assert_eq!(
        command
            .get_envs()
            .filter(|(_, value)| value.is_some())
            .count(),
        1
    );
    assert!(command
        .get_envs()
        .any(|(key, value)| key == "PGSERVICE" && value.is_none()));
}

#[test]
fn the_query_quotes_the_schema_and_never_scans_it_for_placeholders() {
    // Every character that needs quoting, and a name that spells a placeholder.
    let query =
        sql::catalog_query_with_search_path("a\"b'c\\d", PostgresCatalogCoverage::Complete, &[]);
    assert!(query.contains("SET LOCAL search_path = pg_catalog, \"a\"\"b'c\\d\";"));
    assert!(query.contains("nspname = E'a\"b''c\\\\d'"));
    let placeholder =
        sql::catalog_query_with_search_path("__COMPLETE__", PostgresCatalogCoverage::Ordering, &[]);
    assert!(placeholder.contains("nspname = E'__COMPLETE__'"));
    assert!(placeholder.contains("'schema', E'__COMPLETE__'"));
    // `pg_catalog` leads the path so the selected schema cannot shadow a built-in.
    assert!(placeholder.contains("search_path = pg_catalog, \"__COMPLETE__\";"));
    assert!(
        placeholder.contains("'coverage', 'ordering'") && !placeholder.contains("__COVERAGE__")
    );
    assert!(query.contains("'coverage', 'complete'") && !query.contains("__SCHEMA__"));
    assert!(query.contains("'currentDatabase', current_database()"));
    assert!(placeholder.contains("'currentDatabase', current_database()"));
    assert!(query.contains("SET LOCAL statement_timeout = '30s';"));
    assert!(query.starts_with("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY;"));
    assert!(query.ends_with("COMMIT;\n"));
    let evidence = sql::catalog_query_with_search_path(
        "public",
        PostgresCatalogCoverage::Complete,
        &["a'b\\c".to_string()],
    );
    assert!(evidence.contains("(E'a''b\\\\c')"));
    assert!(!evidence.contains("__SEARCH_PATH_EVIDENCE__"));
}

#[test]
fn options_default_to_complete_coverage_and_reject_unknown_values() {
    let parse = |json: &str| serde_json::from_str::<PostgresCatalogOptions>(json);
    let default = parse(r#"{"connectionEnv": "DATABASE_URL", "schema": "public"}"#).unwrap();
    assert_eq!(default.coverage, PostgresCatalogCoverage::Complete);
    assert!(default.search_path_schemas.is_empty());
    let scoped =
        parse(r#"{"connectionEnv":"A","schema":"s","searchPathSchemas":["audit"]}"#).unwrap();
    assert_eq!(scoped.search_path_schemas, ["audit"]);
    let ordering =
        parse(r#"{"connectionEnv": "A", "schema": "s", "coverage": "ordering"}"#).unwrap();
    assert_eq!(ordering.coverage, PostgresCatalogCoverage::Ordering);
    assert!(parse(r#"{"connectionEnv": "A", "schema": "s", "coverage": "partial"}"#).is_err());
    assert!(parse(r#"{"connectionEnv": "A", "schema": "s", "extra": 1}"#).is_err());
    assert_eq!(
        serde_json::to_value(&ordering).unwrap()["coverage"],
        "ordering"
    );
}
