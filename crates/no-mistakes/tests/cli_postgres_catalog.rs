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
    let run = |schema: &str| {
        Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args([
                "postgres",
                "catalog",
                "--connection-env",
                "NO_MISTAKES_TEST_POSTGRES_URL",
                "--schema",
                schema,
                "--output",
            ])
            .env("PGOPTIONS", "-cstandard_conforming_strings=off")
            .arg(&output)
            .output()
            .unwrap()
    };
    assert!(run("pg_catalog").status.success());
    let relative = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "postgres",
            "catalog",
            "--connection-env",
            "NO_MISTAKES_TEST_POSTGRES_URL",
            "--schema",
            "pg_catalog",
            "--output",
            "catalog.json",
        ])
        .current_dir(directory.path())
        .output()
        .unwrap();
    assert!(relative.status.success());
    let original = std::fs::read(&output).unwrap();
    let catalog: serde_json::Value = serde_json::from_slice(&original).unwrap();
    assert_eq!(catalog["coverage"], "ordering");
    assert_eq!(catalog["schema"], "pg_catalog");
    assert!(catalog["tables"].get("pg_class").is_some());
    assert!(run("pg_catalog").status.success());
    assert_eq!(std::fs::read(&output).unwrap(), original);
    let failure = run("missing\\' schema");
    assert!(!failure.status.success());
    assert_eq!(std::fs::read(&output).unwrap(), original);
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
