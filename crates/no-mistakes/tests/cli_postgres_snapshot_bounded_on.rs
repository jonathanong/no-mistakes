use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use serde_json::Value;
use std::process::Command;

fn root() -> std::path::PathBuf {
    rules_path("postgres-bounded-statements", "fixture/bounded-on", "")
}

fn report(config: &str, exit: i32) -> Value {
    let root = root();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(config))
        .args(["--include-sql-verdicts", "--format", "json"])
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(exit), "{report}");
    report
}

fn findings_at(report: &Value, file: &str) -> Vec<(u64, String)> {
    let mut findings = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| {
            finding["file"]
                .as_str()
                .is_some_and(|path| path.ends_with(file))
        })
        .map(|finding| {
            (
                finding["line"].as_u64().unwrap(),
                finding["target"].as_str().unwrap().to_string(),
            )
        })
        .collect::<Vec<_>>();
    findings.sort();
    findings
}

#[test]
fn postgres_bounded_statements_on_clause_roots_check_json_snapshot() {
    let fixture = root();
    assert_saved_fixture(
        "postgres_bounded_statements_on_clause_roots",
        &[
            ("lookup_roots", fixture.clone(), fixture.join("valid.yml")),
            ("advisory", fixture.clone(), fixture.join("advisory.yml")),
            ("embedded", fixture.clone(), fixture.join("embedded.yml")),
        ],
    );
    let report = report("valid.yml", 1);

    for file in [
        "sql/item-feed-topic.sql",
        "sql/account-authorization.sql",
        "sql/participant-user.sql",
    ] {
        assert!(findings_at(&report, file).is_empty(), "{file}: {report}");
    }
    assert_eq!(
        findings_at(&report, "sql/non-key.sql"),
        [(3, "table:authorizations".to_string())],
        "a non-key join is not bounded by its source: {report}"
    );
    assert_eq!(
        findings_at(&report, "sql/unbounded-chain.sql"),
        [
            (2, "table:items".to_string()),
            (3, "table:feeds".to_string()),
            (4, "table:topics".to_string()),
        ],
        "an unbounded key-join chain reports every unbounded relation: {report}"
    );
    let advisory = self::report("advisory.yml", 0);
    assert!(advisory["rules"].as_array().unwrap().is_empty());
    let findings = advisory["advisories"].as_array().unwrap();
    assert_eq!(findings.len(), 2, "{advisory}");
    for table in ["items", "feeds"] {
        assert!(
            findings.iter().any(|finding| {
                finding["target"] == format!("table:{table}")
                    && finding["message"]
                        .as_str()
                        .unwrap()
                        .contains(&format!("SELECT can return every row of {table}"))
            }),
            "{advisory}"
        );
    }
    let embedded = self::report("embedded.yml", 0);
    assert!(embedded["rules"].as_array().unwrap().is_empty());
    assert!(embedded["advisories"].as_array().unwrap().is_empty());
    let selects = embedded["sqlVerdicts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|verdict| verdict["subcheck"] == "select")
        .collect::<Vec<_>>();
    assert_eq!(selects.len(), 3, "{embedded}");
    assert!(
        selects.iter().all(|verdict| {
            verdict["status"] == "proven-clear" && verdict["evidence"] == "exact"
        }),
        "{embedded}"
    );
}
