use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use serde_json::Value;
use std::process::Command;

fn root() -> std::path::PathBuf {
    rules_path("postgres-bounded-statements", "fixture/root-proofs", "")
}

fn report(config: &str) -> Value {
    let root = root();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(config))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "{report}"
    );
    report
}

fn targets(report: &Value, file: &str) -> Vec<String> {
    let mut targets = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| {
            finding["file"]
                .as_str()
                .is_some_and(|path| path.ends_with(file))
        })
        .map(|finding| finding["target"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    targets.sort();
    targets
}

#[test]
fn postgres_bounded_statements_root_proofs_check_json_snapshot() {
    let fixture = root();
    assert_saved_fixture(
        "postgres_bounded_statements_root_proofs",
        &[
            (
                "complete_source_key",
                fixture.clone(),
                fixture.join("valid.yml"),
            ),
            (
                "missing_source_key",
                fixture.clone(),
                fixture.join("invalid.yml"),
            ),
        ],
    );
    let complete = report("valid.yml");
    let incomplete = report("invalid.yml");
    assert_eq!(
        targets(&complete, "sql/participants.sql"),
        ["table:participants"],
        "only the composite-key participant root remains: {complete}"
    );
    assert!(
        targets(&complete, "sql/cte-sources.sql").is_empty(),
        "{complete}"
    );
    assert_eq!(
        targets(&incomplete, "sql/participants.sql"),
        ["table:participants"],
        "the joined conversation is bounded while the participant composite key is incomplete: {incomplete}"
    );
    assert_eq!(
        targets(&incomplete, "sql/cte-sources.sql"),
        ["table:feeds", "table:sources", "table:topics"],
        "only relations lacking a proof should report: {incomplete}"
    );
    assert_eq!(
        targets(&complete, "sql/controls.sql"),
        [
            "table:items",
            "table:items",
            "table:items",
            "table:items",
            "table:items",
            "table:participants",
            "table:participants"
        ],
        "incomplete and non-singleton proofs remain roots: {complete}"
    );
    assert_eq!(
        targets(&complete, "sql/constant-chain.sql"),
        ["table:participants", "table:sources"]
    );
    assert_eq!(
        targets(&complete, "sql/distinct-singleton.sql"),
        vec!["table:items"; 6]
    );
    let recursive: Vec<_> = complete["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["file"] == "sql/recursive-distinct.sql")
        .map(|finding| {
            (
                finding["line"].as_u64().unwrap(),
                finding["target"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(recursive, [(4, "table:items")]);
    let unsafe_distinct: Vec<_> = complete["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["file"] == "sql/distinct-unsafe.sql")
        .map(|finding| {
            (
                finding["line"].as_u64().unwrap(),
                finding["target"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(unsafe_distinct, [(1, "table:items"), (3, "table:items")]);
    assert_eq!(complete["warnings"], serde_json::json!([]), "{complete}");
    assert_eq!(
        incomplete["warnings"],
        serde_json::json!([]),
        "{incomplete}"
    );
}
