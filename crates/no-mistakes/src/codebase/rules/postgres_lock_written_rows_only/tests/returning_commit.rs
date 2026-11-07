use super::*;
#[test]
fn returning_commit_preserves_fallthrough_transaction_and_helper_effects() {
    let findings = check(&fixture("returning-commit"), &config()).unwrap();
    assert!(
        findings.iter().all(|finding| finding.line > 14),
        "{findings:?}"
    );
    assert!(
        findings.iter().any(|finding| finding.line == 17),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.line == 38 && finding.target.as_deref() == Some("unanalyzable")),
        "{findings:?}"
    );
}
