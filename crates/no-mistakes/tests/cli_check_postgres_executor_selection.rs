use std::path::PathBuf;
use std::process::Command;

/// Every rule that scans executor calls. Each has a fixture config that sets
/// neither `importSpecifier` nor `executorNames`.
const EXECUTOR_RULES: [&str; 11] = [
    "postgres-bounded-statements",
    "postgres-explicit-columns",
    "postgres-generated-column-predicates",
    "postgres-no-generated-column-writes",
    "postgres-required-predicates",
    "postgres-sql-shape-policy",
    "postgres-no-offset",
    "postgres-conflict-ordering",
    "postgres-lock-ordering",
    "postgres-idempotent-insert",
    "postgres-require-query-annotation",
];

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/postgres/executor-selection/fixture"),
    )
}

#[test]
fn every_executor_rule_fails_check_when_no_executor_is_selected() {
    let root = fixture();
    for rule in EXECUTOR_RULES {
        for format in ["human", "json"] {
            let out = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                .args(["check", "--format", format, "--root"])
                .arg(&root)
                .arg("--config")
                .arg(root.join(format!("{rule}.yml")))
                .output()
                .unwrap();
            let stderr = String::from_utf8_lossy(&out.stderr);
            assert_eq!(out.status.code(), Some(2), "{rule} {format}: {stderr}");
            assert!(out.stdout.is_empty(), "{rule} {format}");
            assert_eq!(
                stderr,
                format!(
                    "error: {rule} option importSpecifier: set importSpecifier (or \
executorNames) to select executor calls; set executorNames: [] to scan only SQL files \
and native SQL (see docs/migrations/explicit-postgres-executors.md)\n"
                ),
                "{rule} {format}"
            );
        }
    }
}
