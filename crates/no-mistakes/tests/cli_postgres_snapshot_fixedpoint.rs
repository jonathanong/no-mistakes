use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use serde_json::Value;
use std::process::Command;

fn root() -> std::path::PathBuf {
    rules_path("postgres-bounded-statements", "fixture/fixedpoint", "")
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

fn assert_findings_at(report: &Value, file: &str, expected: &[(u64, &str)]) {
    let mut actual = report["rules"]
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
    actual.sort();
    let mut expected = expected
        .iter()
        .map(|(line, target)| (*line, (*target).to_string()))
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(actual, expected, "{file}: {report}");
}

#[test]
fn postgres_bounded_statements_fixedpoint_check_json_snapshot() {
    let fixture = root();
    assert_saved_fixture(
        "postgres_bounded_statements_fixedpoint",
        &[("lookup_roots", fixture.clone(), fixture.join("valid.yml"))],
    );
    let report = report();

    assert_findings_at(
        &report,
        "sql/conjuncts.sql",
        &[
            (2, "table:items"),
            (6, "table:items"),
            (11, "table:items"),
            (20, "table:items"),
        ],
    );
    assert_findings_at(
        &report,
        "sql/derived.sql",
        &[
            (3, "table:items"),
            (10, "table:items"),
            (14, "table:items"),
            (23, "table:items"),
            (27, "table:items"),
            (30, "table:items"),
        ],
    );
    assert_findings_at(
        &report,
        "sql/must-report.sql",
        &[
            (1, "table:items"),
            (3, "table:items"),
            (5, "table:items"),
            (7, "table:items"),
            (12, "table:items"),
            (16, "table:items"),
            (19, "table:items"),
            (1, "table:users"),
            (3, "table:users"),
            (5, "table:users"),
            (7, "table:users"),
            (16, "table:users"),
        ],
    );
    assert_findings_at(
        &report,
        "sql/recursive-conjuncts.sql",
        &[(4, "table:items")],
    );
    assert_findings_at(
        &report,
        "sql/recursive-transitive.sql",
        &[(4, "table:items")],
    );
    assert_findings_at(
        &report,
        "sql/transitive.sql",
        &[
            (2, "table:items"),
            (9, "table:items"),
            (12, "table:items"),
            (17, "table:items"),
            (22, "table:items"),
        ],
    );
    assert_findings_at(
        &report,
        "sql/unproven-drivers.sql",
        &[(1, "table:items"), (3, "table:items"), (6, "table:users")],
    );
}
