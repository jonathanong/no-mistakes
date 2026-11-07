use super::*;

#[test]
fn helper_summaries_preserve_forwarding_and_reject_opaque_or_unbounded_effects() {
    let root = fixture("helper-summary-boundaries");
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let findings = check(&root, &config()).unwrap();
    let mut checked = 0;
    for (index, line) in source.lines().enumerate() {
        let expected = if line.contains("expect-clean") {
            false
        } else if line.contains("expect-unresolved") {
            true
        } else {
            continue;
        };
        let actual = findings
            .iter()
            .filter(|finding| finding.line == index + 1)
            .collect::<Vec<_>>();
        assert_eq!(
            actual
                .iter()
                .any(|finding| finding.target.as_deref() == Some("unanalyzable")),
            expected,
            "line {}: {actual:?}; all findings: {findings:?}",
            index + 1
        );
        if !expected {
            assert!(actual.is_empty(), "line {}: {actual:?}", index + 1);
        }
        checked += 1;
    }
    assert_eq!(checked, 11);
}
