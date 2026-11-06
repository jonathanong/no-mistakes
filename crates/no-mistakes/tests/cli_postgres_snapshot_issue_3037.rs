use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn repo_rel(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
}

fn findings(config: &str) -> Vec<(String, String)> {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/fixture/issue-3037");
    let config = rules_path("postgres-bounded-statements", "fixture", config);
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .current_dir(&root)
        .args(["--jobs", "1", "check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(&config)
        .args(["--format", "json"])
        .output()
        .expect("run no-mistakes check");
    let report: Value = serde_json::from_slice(&output.stdout).expect("check JSON report");
    assert_eq!(
        output.status.code(),
        Some(if config.ends_with("cte-valid.yml") {
            0
        } else {
            1
        }),
        "{report}"
    );
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    let mut findings: Vec<_> = report["rules"]
        .as_array()
        .expect("rules array")
        .iter()
        .map(|finding| {
            (
                finding["file"].as_str().expect("finding file").to_string(),
                finding["target"]
                    .as_str()
                    .expect("finding target")
                    .to_string(),
            )
        })
        .collect();
    findings.sort();
    findings
}

#[test]
fn postgres_bounded_statements_issue_3037_check_json_snapshot() {
    let root = "test-cases/rules/postgres-bounded-statements/fixture/issue-3037";
    assert_saved_fixture(
        "postgres_bounded_statements_issue_3037",
        &[
            (
                "cte_nonkey_join_chain",
                repo_rel(root),
                rules_path(
                    "postgres-bounded-statements",
                    "fixture",
                    "issue-3037/cte-valid.yml",
                ),
            ),
            (
                "multirow_roots_and_controls",
                repo_rel(root),
                rules_path(
                    "postgres-bounded-statements",
                    "fixture",
                    "issue-3037/valid.yml",
                ),
            ),
        ],
    );
    assert!(findings("issue-3037/cte-valid.yml").is_empty());
    assert_eq!(
        findings("issue-3037/valid.yml"),
        vec![
            (
                "sql/bounded_nonkey_key_chain.sql".into(),
                "table:posts".into()
            ),
            (
                "sql/bounded_nonkey_key_chain.sql".into(),
                "table:sources".into()
            ),
            ("sql/unbounded_key_join.sql".into(), "table:posts".into()),
            ("sql/unbounded_key_target.sql".into(), "table:posts".into()),
            (
                "sql/unbounded_key_target.sql".into(),
                "table:sources".into()
            ),
        ]
    );
}
