use super::check_json_clean::assert_saved_fixture_clean;
use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::fs::read_to_string;
use std::process::Command;

const RULE: &str = "postgres-grouping-not-aggregation";

#[test]
fn postgres_grouping_not_aggregation_check_json_snapshot() {
    let root = rules_path(RULE, "fixture", "");
    let cases = [
        ("catalog", root.clone(), root.join(".no-mistakes.yml")),
        ("no_catalog", root.clone(), root.join("no-catalog.yml")),
        ("disabled", root.clone(), root.join("disabled.yml")),
    ];
    assert_saved_fixture("postgres_grouping_not_aggregation", &cases[..1]);
    assert_saved_fixture_clean(
        "postgres_grouping_not_aggregation_clean_controls",
        &cases[1..],
    );
}

#[test]
fn postgres_grouping_not_aggregation_reports_only_marked_sound_controls() {
    let root = rules_path(RULE, "fixture", "");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["rules"],
        serde_json::json!([]),
        "the rule is advisory only"
    );
    let findings = report["advisories"].as_array().unwrap();
    let positive = [
        (
            "raw/shape-controls.sql",
            "raw-all-of-distinct",
            "all-of-candidate-pool",
        ),
        (
            "raw/shape-controls.sql",
            "raw-all-of-count-star",
            "all-of-candidate-pool",
        ),
        (
            "raw/shape-controls.sql",
            "raw-unique-composite-distinct",
            "distinct-over-unique-key",
        ),
        (
            "raw/shape-controls.sql",
            "raw-unique-composite-group",
            "distinct-over-unique-key",
        ),
        (
            "raw/shape-controls.sql",
            "raw-unique-primary-key",
            "distinct-over-unique-key",
        ),
        (
            "raw/shape-controls.sql",
            "raw-unique-count-distinct",
            "distinct-over-unique-key",
        ),
        (
            "raw/shape-controls.sql",
            "raw-unique-partial",
            "distinct-over-unique-key",
        ),
        (
            "raw/shape-controls.sql",
            "raw-unique-distinct-on-join",
            "distinct-over-unique-key",
        ),
        (
            "raw/shape-controls.sql",
            "raw-union-derived",
            "distinct-over-union",
        ),
        (
            "raw/shape-controls.sql",
            "raw-union-arm-left",
            "distinct-over-union",
        ),
        (
            "raw/shape-controls.sql",
            "raw-union-arm-right",
            "distinct-over-union",
        ),
        (
            "raw/shape-controls.sql",
            "raw-union-cte",
            "distinct-over-union",
        ),
        (
            "raw/shape-controls.sql",
            "raw-filter-derived-join",
            "distinct-join-filter",
        ),
        (
            "raw/shape-controls.sql",
            "raw-filter-all-membership",
            "distinct-join-filter",
        ),
        (
            "raw/shape-controls.sql",
            "raw-filter-cte-in",
            "distinct-join-filter",
        ),
        (
            "raw/shape-controls.sql",
            "raw-filter-cte-not-exists",
            "distinct-join-filter",
        ),
        (
            "raw/shape-controls.sql",
            "raw-filter-count-distinct",
            "distinct-join-filter",
        ),
        (
            "raw/shape-controls.sql",
            "raw-count-positive",
            "aggregate-existence-test",
        ),
        (
            "raw/shape-controls.sql",
            "raw-count-zero",
            "aggregate-existence-test",
        ),
        (
            "raw/shape-controls.sql",
            "raw-count-direct-positive",
            "aggregate-existence-test",
        ),
        (
            "raw/shape-controls.sql",
            "raw-count-direct-zero",
            "aggregate-existence-test",
        ),
        (
            "raw/shape-controls.sql",
            "raw-aggregate-not-exists-max",
            "aggregate-existence-test",
        ),
        (
            "src/shape-controls.ts",
            "embedded-all-of-distinct",
            "all-of-candidate-pool",
        ),
        (
            "src/shape-controls.ts",
            "embedded-unique-composite-distinct",
            "distinct-over-unique-key",
        ),
        (
            "src/shape-controls.ts",
            "embedded-unique-composite-group",
            "distinct-over-unique-key",
        ),
        (
            "src/shape-controls.ts",
            "embedded-unique-primary-key",
            "distinct-over-unique-key",
        ),
        (
            "src/shape-controls.ts",
            "embedded-unique-count-distinct",
            "distinct-over-unique-key",
        ),
        (
            "src/shape-controls.ts",
            "embedded-unique-partial",
            "distinct-over-unique-key",
        ),
        (
            "src/shape-controls.ts",
            "embedded-unique-distinct-on-join",
            "distinct-over-unique-key",
        ),
        (
            "src/shape-controls.ts",
            "embedded-union-derived",
            "distinct-over-union",
        ),
        (
            "src/shape-controls.ts",
            "embedded-union-arm-left",
            "distinct-over-union",
        ),
        (
            "src/shape-controls.ts",
            "embedded-union-arm-right",
            "distinct-over-union",
        ),
        (
            "src/shape-controls.ts",
            "embedded-union-cte",
            "distinct-over-union",
        ),
        (
            "src/shape-controls.ts",
            "embedded-filter-derived-join",
            "distinct-join-filter",
        ),
        (
            "src/shape-controls.ts",
            "embedded-filter-all-membership",
            "distinct-join-filter",
        ),
        (
            "src/shape-controls.ts",
            "embedded-filter-cte-in",
            "distinct-join-filter",
        ),
        (
            "src/shape-controls.ts",
            "embedded-filter-cte-not-exists",
            "distinct-join-filter",
        ),
        (
            "src/shape-controls.ts",
            "embedded-filter-count-distinct",
            "distinct-join-filter",
        ),
        (
            "src/shape-controls.ts",
            "embedded-count-positive",
            "aggregate-existence-test",
        ),
        (
            "src/shape-controls.ts",
            "embedded-count-zero",
            "aggregate-existence-test",
        ),
        (
            "src/shape-controls.ts",
            "embedded-count-direct-positive",
            "aggregate-existence-test",
        ),
        (
            "src/shape-controls.ts",
            "embedded-count-direct-zero",
            "aggregate-existence-test",
        ),
        (
            "src/shape-controls.ts",
            "embedded-aggregate-not-exists-max",
            "aggregate-existence-test",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-all-of-distinct",
            "all-of-candidate-pool",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-all-of-count-star",
            "all-of-candidate-pool",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-unique-partial",
            "distinct-over-unique-key",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-unique-count-distinct",
            "distinct-over-unique-key",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-unique-composite-group",
            "distinct-over-unique-key",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-unique-distinct-on-join",
            "distinct-over-unique-key",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-union",
            "distinct-over-union",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-union-cte",
            "distinct-over-union",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-union-arm-left",
            "distinct-over-union",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-union-arm-right",
            "distinct-over-union",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-filter-join",
            "distinct-join-filter",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-filter-cte-not-exists",
            "distinct-join-filter",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-count-positive",
            "aggregate-existence-test",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-count-zero",
            "aggregate-existence-test",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-count-direct-positive",
            "aggregate-existence-test",
        ),
        (
            "src/shape-controls.ts",
            "embedded-append-count-direct-zero",
            "aggregate-existence-test",
        ),
    ];
    for (file, marker, code) in positive {
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
    assert_eq!(
        findings.len(),
        positive.len(),
        "unmarked findings mean a negative control fired"
    );
    for file in ["raw/shape-controls.sql", "src/shape-controls.ts"] {
        let source = read_to_string(root.join(file)).unwrap();
        for (index, source_line) in source
            .lines()
            .enumerate()
            .filter(|(_, line)| line.contains("CASE:") && line.contains("negative"))
        {
            let line = index + 1;
            assert!(
                findings.iter().all(|finding| {
                    finding["file"]
                        .as_str()
                        .is_none_or(|path| !path.ends_with(file) || finding["line"] != line)
                }),
                "negative control reported at {file}:{line}: {findings:#?}"
            );
        }
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

#[test]
fn postgres_grouping_not_aggregation_legacy_control_matches_scope() {
    let root = rules_path(RULE, "fixture", "");
    assert_saved_fixture_clean(
        "postgres_grouping_not_aggregation_legacy_control",
        &[("legacy", root.clone(), root.join("legacy-control.yml"))],
    );
}

#[test]
fn postgres_grouping_not_aggregation_rejects_blocking_severity_and_fail_option() {
    let root = rules_path(RULE, "fixture", "");
    for (config, expected) in [
        ("reject-error-severity.yml", "advisory"),
        ("reject-fail-severity.yml", "fail"),
        ("reject-fail-option.yml", "fail"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(config))
            .args(["--format", "json"])
            .output()
            .unwrap();
        assert!(!output.status.success(), "{config} must be rejected");
        let output = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.to_ascii_lowercase().contains(expected),
            "{config} should explain its invalid setting: {output}"
        );
    }
}
