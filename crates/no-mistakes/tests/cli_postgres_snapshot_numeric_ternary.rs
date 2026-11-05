use super::check_json_snapshot::assert_saved_fixture;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

const CONSUMERS: &[&str] = &[
    "postgres-bounded-statements",
    "postgres-explicit-columns",
    "postgres-required-predicates",
    "postgres-generated-column-predicates",
    "postgres-no-transformed-uuid-predicates",
    "postgres-idempotent-insert",
    "postgres-sql-statement-policy",
    "postgres-no-test-state-mutation",
    "postgres-no-offset",
    "postgres-no-generated-column-writes",
    "postgres-conflict-ordering",
    "postgres-lock-ordering",
    "postgres-sql-shape-policy",
    "postgres-require-query-annotation",
    "postgres-json-relation-ids",
    "postgres-no-unbounded-sql-identifier",
    "postgres-no-mutable-sql-registry",
];

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres/numeric-ternary-consumers"),
    )
}

#[test]
fn postgres_numeric_ternary_consumers_check_json_snapshot() {
    let root = fixture();
    let cases = [
        "numeric-ignore",
        "string-ignore",
        "numeric-fail",
        "string-fail",
    ]
    .map(|name| (name, root.clone(), root.join(format!("{name}.yml"))));
    assert_saved_fixture("postgres_numeric_ternary_consumers", &cases);
}

fn report(kind: &str, policy: &str) -> Value {
    let root = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .current_dir(&root)
        .args(["--jobs", "1", "check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(format!("{kind}-{policy}.yml")))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn findings(report: &Value, kind: &str) -> Vec<Value> {
    let mut findings = Vec::new();
    let expected_file = format!("{kind}.ts");
    for category in ["rules", "advisories"] {
        for finding in report[category].as_array().unwrap() {
            if finding["file"].as_str() != Some(expected_file.as_str()) {
                continue;
            }
            findings.push(serde_json::json!({
                "category": category,
                "rule": finding["rule"],
                "line": finding["line"],
                "target": finding["target"],
                "message": finding["message"].as_str().unwrap()
                    .replace(&format!("{kind}.ts"), "query.ts"),
            }));
        }
    }
    findings.sort_by_key(Value::to_string);
    findings
}

#[test]
fn numeric_and_string_ternaries_preserve_all_consumer_findings() {
    for policy in ["ignore", "fail"] {
        let numeric = findings(&report("numeric", policy), "numeric");
        let string = findings(&report("string", policy), "string");
        assert_eq!(numeric, string, "consumer parity under {policy}");
        let rules = numeric
            .iter()
            .map(|finding| finding["rule"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(rules, CONSUMERS.iter().copied().collect(), "{policy}");
        assert!(
            numeric.iter().any(|finding| {
                finding["rule"] == "postgres-lock-ordering" && finding["target"] == "unparseable"
            }),
            "invalid locking SQL stays reportable under {policy}: {numeric:?}"
        );
    }
}
