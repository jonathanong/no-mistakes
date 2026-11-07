use super::check_json_clean::assert_saved_fixture_clean;
use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_identifier_catalog_repros_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-catalogs",
    );
    for shape in ["producer", "record", "join"] {
        assert_saved_fixture_clean(
            &format!("postgres_identifier_catalog_{shape}"),
            &[(
                shape,
                root.clone(),
                root.join(format!(".no-mistakes-{shape}.yml")),
            )],
        );
    }
    for shape in ["producer-off", "producer-negative", "negative"] {
        assert_saved_fixture(
            &format!("postgres_identifier_catalog_{shape}"),
            &[(
                shape,
                root.clone(),
                root.join(format!(".no-mistakes-{shape}.yml")),
            )],
        );
    }
}

#[test]
fn postgres_workspace_identifier_sources_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-workspace-producer",
    );
    for shape in [
        "origin",
        "callsite",
        "overlap",
        "name",
        "unused-ignore",
        "guarded",
    ] {
        assert_saved_fixture_clean(
            &format!("postgres_workspace_identifier_sources_{shape}"),
            &[(
                shape,
                root.clone(),
                root.join(format!(".no-mistakes-{shape}.yml")),
            )],
        );
    }
    for shape in [
        "unused",
        "wrong",
        "typo",
        "member",
        "member-wrapped",
        "opaque",
        "value-only",
        "unused-advise",
        "unchecked",
    ] {
        assert_saved_fixture(
            &format!("postgres_workspace_identifier_sources_{shape}"),
            &[(
                shape,
                root.clone(),
                root.join(format!(".no-mistakes-{shape}.yml")),
            )],
        );
    }
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes-unchecked.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    for (line, target) in [
        (5, "table"),
        (10, "alias"),
        (16, "alias"),
        (23, "alias"),
        (29, "literal"),
    ] {
        assert!(
            report["rules"].as_array().unwrap().iter().any(|finding| {
                finding["rule"] == "postgres-no-unbounded-sql-identifier"
                    && finding["file"] == "apps/worker/unchecked.ts"
                    && finding["line"] == line
                    && finding["target"] == target
            }),
            "unchecked catalog-type reference {target} at {line} must be reported: {report}"
        );
    }
}
