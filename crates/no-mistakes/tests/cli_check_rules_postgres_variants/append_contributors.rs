use super::check_with_suppressed;

#[test]
fn disabled_executions_own_only_the_append_sites_that_contributed_their_sql() {
    for (name, active, suppressed_line) in [
        ("complete-suppression-reuse", 1, 5),
        ("complete-suppression-owned", 0, 7),
        ("complete-suppression-nested", 1, 7),
        ("complete-suppression-branch-assigned", 0, 9),
        ("complete-suppression-folded", 0, 8),
        ("complete-suppression-fluent", 0, 4),
    ] {
        let report = check_with_suppressed("postgres-sql-shape-policy", name, true);
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), active, "{name}: {report}");
        for finding in findings {
            assert_eq!(finding["line"], 2, "{name}: {report}");
            assert_eq!(
                finding["target"], "banned-function-call",
                "{name}: {report}"
            );
        }
        let suppressed = report["suppressed"].as_array().unwrap();
        assert_eq!(suppressed.len(), 1, "{name}: {report}");
        assert_eq!(suppressed[0]["line"], suppressed_line, "{name}: {report}");
    }
}
