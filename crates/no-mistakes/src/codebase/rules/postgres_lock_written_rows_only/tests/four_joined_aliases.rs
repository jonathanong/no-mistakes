use super::*;

#[test]
fn four_joined_lock_aliases_are_checked_against_each_write_relation() {
    let root = fixture("four-joined-aliases");
    let findings = check(&root, &config()).unwrap();
    let mut partial = findings
        .iter()
        .filter(|finding| finding.line == 20)
        .map(|finding| {
            assert_eq!(finding.target.as_deref(), Some(CHECKS[0]), "{finding:?}");
            finding.message.clone()
        })
        .collect::<Vec<_>>();
    partial.sort();
    assert_eq!(partial.len(), 2, "{findings:?}");
    assert!(partial.iter().any(|message| message.contains("in reviews")));
    assert!(
        partial
            .iter()
            .any(|message| message.contains("in decisions"))
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.line == 33 && finding.target.as_deref() == Some("unanalyzable")),
        "{findings:?}"
    );
    assert_eq!(findings.len(), 3, "{findings:?}");
    assert!(findings.iter().all(|finding| finding.line != 5));
    assert!(findings.iter().all(|finding| finding.line < 43));
}
