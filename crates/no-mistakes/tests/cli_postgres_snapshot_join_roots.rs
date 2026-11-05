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
        &[(
            "join_key_roots",
            fixture.clone(),
            fixture.join("valid.yml"),
        )],
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
            "table:public.genericitems",
            "table:public.genericitems",
            "table:public.posts",
            "table:public.posts",
        ],
        "{report}"
    );
    assert_eq!(
        targets_at("sql/unique-composite.sql"),
        ["table:public.genericitems", "table:public.posts"],
        "{report}"
    );
    assert_eq!(
        targets_at("sql/unproven-keys.sql"),
        [
            "table:public.genericitems",
            "table:public.genericitems",
            "table:public.genericitems",
            "table:public.posts",
            "table:public.posts",
            "table:public.posts",
            "table:public.users",
            "table:public.users",
            "table:public.users",
            "table:public.users",
        ],
        "{report}"
    );
    assert_eq!(
        targets_at("sql/join-direction-controls.sql"),
        [
            "table:public.genericitems",
            "table:public.genericitems",
            "table:public.genericitems",
            "table:public.posts",
            "table:public.posts",
            "table:public.topics",
            "table:public.users",
        ],
        "{report}"
    );
    assert!(
        targets_at("sql/cte-derived-self.sql")
            .iter()
            .all(|target| target == "table:public.posts"),
        "a lookup-side relation must not be promoted to a root: {report}"
    );
    assert!(
        !targets_at("sql/cte-derived-self.sql").is_empty(),
        "nested read roots must be retained: {report}"
    );
    assert!(targets_at("sql/limited-control.sql").is_empty(), "{report}");
}
