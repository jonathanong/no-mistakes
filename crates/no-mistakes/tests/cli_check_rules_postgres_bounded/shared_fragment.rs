use super::{check_in, findings_of, fixture_dir};

fn lines_of(config: &str) -> Vec<(String, u64)> {
    let root = fixture_dir("variants/shared-fragment");
    let mut found: Vec<_> = findings_of(check_in(&root, config))
        .into_iter()
        .map(|(file, line, _)| (file, line))
        .collect();
    found.sort();
    found
}

#[test]
fn statements_appending_a_shared_fragment_are_reported_at_each_caller() {
    assert_eq!(
        lines_of(".no-mistakes.yml"),
        [
            ("src/first.ts".to_string(), 5),
            ("src/full-from.ts".to_string(), 9),
            ("src/full-from.ts".to_string(), 15),
            ("src/full-from.ts".to_string(), 21),
            ("src/full-from.ts".to_string(), 38),
            ("src/full-from.ts".to_string(), 42),
            ("src/full-from.ts".to_string(), 51),
            ("src/full-from.ts".to_string(), 59),
            ("src/second.ts".to_string(), 5),
            ("src/third.ts".to_string(), 7)
        ]
    );
}

#[test]
fn a_caller_directive_suppresses_only_that_callers_statement() {
    let found = lines_of(".no-mistakes.yml");
    assert!(found.iter().all(|(file, _)| file != "src/suppressed.ts"));
    assert!(found.iter().all(|(file, _)| file != "src/fragment.ts"));
}

#[test]
fn a_caller_line_directive_suppresses_a_finding_relocated_to_the_executing_call() {
    let found = lines_of(".no-mistakes.yml");
    assert!(found.iter().all(|(file, _)| file != "src/appended.ts"));
    let output = check_in(&fixture_dir("variants/shared-fragment"), ".no-mistakes.yml");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["advisories"]
            .as_array()
            .unwrap()
            .iter()
            .all(|advisory| advisory["file"] != "src/appended.ts"),
        "{report}"
    );
}

#[test]
fn shared_fragment_check_json_snapshot() {
    let root = fixture_dir("variants/shared-fragment");
    crate::check_json_snapshot::assert_saved_fixture(
        "postgres_bounded_shared_fragment",
        &[("callers", root.clone(), root.join(".no-mistakes.yml"))],
    );
}

#[test]
fn same_file_full_from_fragments_keep_both_callers_and_ignore_a_suppressed_opening_root() {
    let found = findings_of(check_in(
        &fixture_dir("variants/shared-fragment"),
        ".no-mistakes.yml",
    ));
    let found: Vec<_> = found
        .into_iter()
        .filter(|(file, _, _)| file == "src/full-from.ts")
        .collect();
    assert_eq!(
        found,
        [
            (
                "src/full-from.ts".to_string(),
                9,
                "table:accounts, table:orders".to_string()
            ),
            (
                "src/full-from.ts".to_string(),
                15,
                "table:accounts, table:orders".to_string()
            ),
            (
                "src/full-from.ts".to_string(),
                21,
                "table:invoices, table:orders".to_string()
            ),
            (
                "src/full-from.ts".to_string(),
                38,
                "table:invoices".to_string()
            ),
            (
                "src/full-from.ts".to_string(),
                42,
                "table:invoices".to_string()
            ),
            (
                "src/full-from.ts".to_string(),
                51,
                "table:invoices".to_string()
            ),
            (
                "src/full-from.ts".to_string(),
                59,
                "table:invoices".to_string()
            ),
        ]
    );
}

#[test]
fn full_from_single_root_callers_do_not_merge_or_override_opening_relations() {
    let root = fixture_dir("variants/shared-fragment");
    let output = check_in(&root, ".no-mistakes.yml");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["rules"].as_array().unwrap();
    for line in [38, 42] {
        let finding = findings
            .iter()
            .find(|finding| finding["file"] == "src/full-from.ts" && finding["line"] == line)
            .unwrap();
        assert_eq!(finding["target"], "table:invoices");
        assert!(finding["message"]
            .as_str()
            .unwrap()
            .contains("includes SQL appended from src/full-from.ts:34"));
    }
    assert!(!findings
        .iter()
        .any(|finding| finding["file"] == "src/full-from.ts" && finding["line"] == 47));
    for line in [51, 59] {
        let finding = findings
            .iter()
            .find(|finding| finding["file"] == "src/full-from.ts" && finding["line"] == line)
            .unwrap();
        assert_eq!(finding["target"], "table:invoices");
        assert!(!finding["message"]
            .as_str()
            .unwrap()
            .contains("includes SQL appended"));
    }
}
