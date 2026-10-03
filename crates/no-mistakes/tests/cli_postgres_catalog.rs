use std::process::Command;

#[test]
fn postgres_catalog_cli_writes_deterministic_owned_json_and_preserves_output_on_failure() {
    if std::env::var_os("NO_MISTAKES_TEST_POSTGRES_URL").is_none() {
        assert!(
            std::env::var_os("CI").is_none(),
            "CI must provide NO_MISTAKES_TEST_POSTGRES_URL"
        );
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("catalog.json");
    let run_with = |schema: &str, coverage: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args([
                "postgres",
                "catalog",
                "--connection-env",
                "NO_MISTAKES_TEST_POSTGRES_URL",
                "--schema",
                schema,
            ])
            .args(coverage)
            .arg("--output")
            .env("PGOPTIONS", "-cstandard_conforming_strings=off")
            .arg(&output)
            .output()
            .unwrap()
    };
    let run = |schema: &str| run_with(schema, &[]);
    // The default is a complete catalog, including every function and view.
    assert!(run("pg_catalog").status.success());
    let original = std::fs::read(&output).unwrap();
    let catalog: serde_json::Value = serde_json::from_slice(&original).unwrap();
    assert_eq!(catalog["coverage"], "complete");
    assert_eq!(catalog["schema"], "pg_catalog");
    assert!(catalog["tables"].get("pg_class").is_some());
    assert!(!catalog["functions"].as_object().unwrap().is_empty());
    assert!(!catalog["views"].as_object().unwrap().is_empty());
    assert!(run("pg_catalog").status.success());
    assert_eq!(std::fs::read(&output).unwrap(), original);
    let failure = run("missing\\' schema");
    assert!(!failure.status.success());
    assert_eq!(std::fs::read(&output).unwrap(), original);
    // Ordering coverage stays available, states itself, and carries no other schema facts.
    assert!(run_with("pg_catalog", &["--coverage", "ordering"])
        .status
        .success());
    let ordering: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    assert_eq!(ordering["coverage"], "ordering");
    assert!(ordering.get("functions").is_none());
    // Publication leaves no temporary file behind, and a relative output path works.
    let relative = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["postgres", "catalog", "--connection-env"])
        .args(["NO_MISTAKES_TEST_POSTGRES_URL", "--schema", "pg_catalog"])
        .args(["--output", "catalog.json"])
        .current_dir(directory.path())
        .output()
        .unwrap();
    assert!(relative.status.success());
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    let unknown = run_with("pg_catalog", &["--coverage", "partial"]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&unknown.stderr).contains("possible values: complete, ordering")
    );
}

#[test]
fn postgres_catalog_cli_reports_safe_connection_and_output_errors() {
    let directory = tempfile::tempdir().unwrap();
    for (connection_env, schema) in [
        ("", "public"),
        ("a=b", "public"),
        ("CATALOG_TEST_CONNECTION", ""),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args([
                "postgres",
                "catalog",
                "--connection-env",
                connection_env,
                "--schema",
                schema,
                "--output",
            ])
            .arg(directory.path())
            .output()
            .unwrap();
        assert!(!output.status.success());
    }
    let run = |url: Option<&str>, path: Option<&str>| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
        command
            .args([
                "postgres",
                "catalog",
                "--connection-env",
                "CATALOG_TEST_CONNECTION",
                "--schema",
                "public",
                "--output",
            ])
            .arg(directory.path());
        command.env_remove("CATALOG_TEST_CONNECTION");
        if let Some(url) = url {
            command.env("CATALOG_TEST_CONNECTION", url);
        }
        if let Some(path) = path {
            command.env("PATH", path);
        }
        command.output().unwrap()
    };
    for output in [
        run(None, None),
        run(Some(""), None),
        run(Some("not-a-postgres-url"), None),
        run(Some("postgresql://user%FF@localhost/db"), None),
        run(Some("postgresql://user:password%FF@localhost/db"), None),
        run(Some("postgresql://localhost/db%FF"), None),
        run(
            Some("postgresql://localhost/db"),
            Some("/missing-no-mistakes-psql"),
        ),
        run(
            Some("postgresql://user:secret-do-not-print@localhost:1/db?connect_timeout=1"),
            None,
        ),
    ] {
        assert!(!output.status.success());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("secret-do-not-print"));
    }
    if let Ok(url) = std::env::var("NO_MISTAKES_TEST_POSTGRES_URL") {
        assert!(
            !run(Some(&url), None).status.success(),
            "a directory cannot be overwritten with generated JSON"
        );
    }
}
