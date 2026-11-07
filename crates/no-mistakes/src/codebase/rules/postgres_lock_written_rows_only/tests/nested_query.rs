use super::*;
#[test]
fn nested_query_helpers_project_parameters_at_known_transaction_calls() {
    let findings = check(&fixture("helper-nested-query"), &config()).unwrap();
    assert!(
        findings
            .iter()
            .all(|finding| ![22, 23].contains(&finding.line)),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.line == 27 && finding.target.as_deref() == Some(CHECKS[0])),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.line == 62 && finding.target.as_deref() == Some("unanalyzable")),
        "{findings:?}"
    );
}
