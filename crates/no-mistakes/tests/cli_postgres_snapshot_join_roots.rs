use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use serde_json::Value;
use std::process::Command;

fn root() -> std::path::PathBuf {
    rules_path("postgres-bounded-statements", "fixture/join-roots", "")
}

fn report() -> Value {
    let root = root();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join("valid.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(1), "{report}");
    report
}

#[test]
fn postgres_bounded_statements_join_roots_check_json_snapshot() {
    // Snapshot counts and targets across PK, unique, composite, partial,
    // nullable, expression, CTE, derived, and self-join cases.
    let fixture = root();
    assert_saved_fixture(
        "postgres_bounded_statements_join_roots",
        &[("join_key_roots", fixture.clone(), fixture.join("valid.yml"))],
    );
    let report = report();
    let findings = report["rules"].as_array().unwrap();
    let targets_at = |file: &str| {
        let mut targets = findings
            .iter()
            .filter(|finding| {
                finding["file"]
                    .as_str()
                    .is_some_and(|actual| actual.ends_with(file))
            })
            .map(|finding| finding["target"].as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        targets.sort();
        targets
    };

    assert_eq!(
        targets_at("sql/pk-left-inner-chain.sql"),
        [
            "table:genericitems",
            "table:genericitems",
            "table:posts",
            "table:posts",
        ],
        "{report}"
    );
    assert_eq!(
        targets_at("sql/unique-composite.sql"),
        ["table:genericitems", "table:posts"],
        "{report}"
    );
    assert_eq!(
        targets_at("sql/unproven-keys.sql"),
        [
            "table:genericitems",
            "table:genericitems",
            "table:genericitems",
            "table:posts",
            "table:posts",
            "table:posts",
            "table:users",
            "table:users",
            "table:users",
        ],
        "{report}"
    );
    assert_eq!(
        targets_at("sql/join-direction-controls.sql"),
        [
            "table:genericitems",
            "table:genericitems",
            "table:genericitems",
            "table:posts",
            "table:posts",
            "table:topics",
            "table:users",
        ],
        "{report}"
    );
    assert_eq!(
        targets_at("sql/cte-derived-self.sql"),
        ["table:posts", "table:posts", "table:posts"],
        "nested reads and the self-join driver must remain roots: {report}"
    );
    assert_eq!(
        targets_at("sql/unproven-drivers.sql"),
        [
            "table:genericitems",
            "table:posts",
            "table:posts",
            "table:posts",
        ],
        "lateral, external, recursive, and opaque nested drivers retain their existing roots: {report}"
    );
    assert!(targets_at("sql/limited-control.sql").is_empty(), "{report}");
}
