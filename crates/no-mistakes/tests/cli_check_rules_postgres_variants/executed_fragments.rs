use super::check_with_suppressed;

#[test]
fn complete_execution_filter_preserves_unrelated_and_unexecuted_fragment_origins() {
    for (name, line, suppressed_line) in [
        ("executed-complete-reuse", 2, 5),
        ("executed-complete-owned", 2, 7),
        ("executed-complete-origin", 3, 8),
        ("executed-complete-helper-reuse", 4, 7),
    ] {
        let report = check_with_suppressed("postgres-sql-shape-policy", name, true);
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{name}: {report}");
        assert_eq!(findings[0]["line"], line, "{name}: {report}");
        assert_eq!(
            findings[0]["target"], "banned-function-call",
            "{name}: {report}"
        );
        let suppressed = report["suppressed"].as_array().unwrap();
        assert_eq!(suppressed.len(), 1, "{name}: {report}");
        assert_eq!(suppressed[0]["line"], suppressed_line, "{name}: {report}");
    }
    let report = check_with_suppressed(
        "postgres-sql-shape-policy",
        "executed-complete-helper-owned",
        true,
    );
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    let suppressed = report["suppressed"].as_array().unwrap();
    assert_eq!(suppressed.len(), 1, "{report}");
    assert_eq!(suppressed[0]["line"], 7, "{report}");
}
