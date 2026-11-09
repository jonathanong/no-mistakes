use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::fs::read_to_string;
use std::process::Command;

const RULE: &str = "postgres-grouping-not-aggregation";

#[test]
fn postgres_grouping_parameter_collision_snapshot() {
    let root = rules_path(RULE, "fixture", "");
    assert_saved_fixture(
        "postgres_grouping_not_aggregation_parameter_collision",
        &[("parameter", root.clone(), root.join("parameter.yml"))],
    );
}

#[test]
fn postgres_grouping_parameter_collision_reports_only_real_column_keys() {
    let root = rules_path(RULE, "fixture", "");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join("parameter.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings: Vec<_> = report["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == RULE)
        .collect();
    let expected: serde_json::Value =
        serde_json::from_str(&read_to_string(root.join("parameter/expected.json")).unwrap())
            .unwrap();
    let expected = expected.as_array().unwrap();
    for case in expected {
        let file = case["file"].as_str().unwrap();
        let marker = case["marker"].as_str().unwrap();
        let code = case["code"].as_str().unwrap();
        let line = line_for_marker(&root, file, marker);
        assert!(
            findings.iter().any(|finding| {
                finding["file"]
                    .as_str()
                    .is_some_and(|path| path.ends_with(file))
                    && finding["line"] == line
                    && finding["message"]
                        .as_str()
                        .is_some_and(|message| message.contains(&format!("[{code}]")))
            }),
            "missing {code} at {file}:{line} ({marker}): {findings:#?}"
        );
    }
    assert_eq!(findings.len(), expected.len(), "unexpected rule finding");
    for (file, marker) in [
        (
            "parameter/raw/parameters.sql",
            "raw-bind-collision-negative",
        ),
        (
            "parameter/src/parameters.ts",
            "embedded-bind-collision-negative",
        ),
        (
            "parameter/src/parameters.ts",
            "append-bind-collision-negative",
        ),
    ] {
        let line = line_for_marker(&root, file, marker);
        assert!(
            findings.iter().all(|finding| {
                finding["file"]
                    .as_str()
                    .is_none_or(|path| !path.ends_with(file) || finding["line"] != line)
            }),
            "parameter was mistaken for a catalog column at {file}:{line}"
        );
    }
}

fn line_for_marker(root: &std::path::Path, file: &str, marker: &str) -> usize {
    read_to_string(root.join(file))
        .unwrap()
        .lines()
        .position(|line| line.contains(marker))
        .map(|index| index + 1)
        .unwrap_or_else(|| panic!("missing fixture marker {marker} in {file}"))
}
