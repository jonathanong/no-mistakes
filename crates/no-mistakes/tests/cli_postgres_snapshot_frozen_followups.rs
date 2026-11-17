use super::check_json_snapshot::assert_saved_fixture_report;
use std::path::PathBuf;

#[test]
fn postgres_closed_identifiers_frozen_followups_check_json_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-closed-identifiers/frozen-followups"),
    );
    let reports = assert_saved_fixture_report(
        "postgres_closed_identifiers_frozen_followups",
        &[("frozen-followups", root.clone(), root.join("config.yml"))],
        &[],
        &[],
    );
    let findings = reports["frozen-followups"]["report"]["rules"]
        .as_array()
        .expect("rule findings array");
    assert_eq!(
        findings.len(),
        18,
        "nine opaque queries, two rules: {findings:?}"
    );
    for file in [
        "tests/shallow-mutation.test.ts",
        "tests/caller-mutation.test.ts",
        "tests/imported-mutator.test.ts",
        "tests/helper-extra-read.test.ts",
        "tests/shadowed-helper.test.ts",
    ] {
        assert_opaque(findings, file, None);
    }
    for line in [30, 31, 32, 33] {
        assert_opaque(findings, "tests/non-sync-helpers.test.ts", Some(line));
    }
    assert_eq!(
        findings
            .iter()
            .filter(|finding| path_ends_with(finding, "tests/non-sync-helpers.test.ts"))
            .count(),
        8,
        "four non-sync helper queries should each produce one finding per rule: {findings:?}"
    );
    for file in [
        "tests/candidate-filtering.test.ts",
        "tests/deep-frozen.test.ts",
        "tests/strings-unrelated.test.ts",
        "tests/lexical-controls.test.ts",
    ] {
        assert!(
            findings
                .iter()
                .all(|finding| !path_ends_with(finding, file)),
            "expected no findings for positive control {file}: {findings:?}"
        );
    }
}

fn assert_opaque(findings: &[serde_json::Value], file: &str, line: Option<u64>) {
    let matching: Vec<_> = findings
        .iter()
        .filter(|finding| {
            path_ends_with(finding, file) && line.is_none_or(|line| finding["line"] == line)
        })
        .collect();
    assert_eq!(
        matching.len(),
        2,
        "expected exactly one opaque finding per configured rule for {file} at {line:?}: {matching:?}"
    );
    for rule in [
        "postgres-no-test-state-mutation",
        "postgres-bounded-statements",
    ] {
        assert!(
            matching.iter().any(|finding| {
                finding["rule"] == rule
                    && finding["target"]
                        == if rule == "postgres-no-test-state-mutation" {
                            "unanalyzable-sql"
                        } else {
                            "unanalyzable"
                        }
            }),
            "expected {rule} unanalyzable finding for {file} at {line:?}: {matching:?}"
        );
    }
}

fn path_ends_with(finding: &serde_json::Value, suffix: &str) -> bool {
    finding["file"]
        .as_str()
        .is_some_and(|file| file.ends_with(suffix))
}
