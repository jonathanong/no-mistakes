use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

const RULE: &str = "postgres-lock-written-rows-only";

#[test]
fn postgres_caller_options_followup_check_json_snapshot() {
    let root = rules_path(RULE, "fixture", "caller-options-followup");
    let report = assert_saved_fixture_report(
        "postgres_caller_options_followup",
        &[
            (
                "with_options",
                root.clone(),
                rules_path(RULE, "configs", "caller-options-followup.yml"),
            ),
            (
                "without_options",
                root,
                rules_path(RULE, "configs", "caller-options-followup-no-options.yml"),
            ),
        ],
        &[],
        &[],
    );
    let findings = report["with_options"]["report"]["rules"]
        .as_array()
        .unwrap();
    assert!(!findings
        .iter()
        .any(|finding| finding["file"] == "src/called.ts"));
    assert!(findings
        .iter()
        .any(|finding| finding["file"] == "src/reassigned.ts"
            && finding["line"] == 8
            && finding["target"] == "unanalyzable"
            && finding["sourceLine"] == "await lockReassigned({ query: tx });"));
    for label in ["with_options", "without_options"] {
        let findings = report[label]["report"]["rules"].as_array().unwrap();
        for (file, line) in [
            ("src/uncalled.ts", 3),
            ("src/uncalled.ts", 6),
            ("src/uncalled.ts", 9),
            ("src/anonymous.ts", 3),
            ("src/exported.ts", 3),
        ] {
            assert!(
                findings.iter().any(|finding| finding["file"] == file
                    && finding["line"] == line
                    && finding["target"] == "row-lock-outside-transaction"),
                "{label}: {findings:?}"
            );
        }
    }
}

#[test]
fn postgres_caller_options_binding_parity_check_json_snapshot() {
    let root = rules_path(RULE, "fixture", "caller-options-followup");
    let report = assert_saved_fixture_report(
        "postgres_caller_options_binding_parity",
        &[
            (
                "with_options",
                root.clone(),
                rules_path(RULE, "configs", "caller-options-binding.yml"),
            ),
            (
                "without_options",
                root,
                rules_path(RULE, "configs", "caller-options-binding-no-options.yml"),
            ),
        ],
        &[],
        &[],
    );
    assert_eq!(report["with_options"], report["without_options"]);
    let findings = report["with_options"]["report"]["rules"]
        .as_array()
        .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0]["file"], "src/binding.ts");
    assert_eq!(findings[0]["line"], 3);
    assert_eq!(findings[0]["target"], "row-lock-outside-transaction");
}
