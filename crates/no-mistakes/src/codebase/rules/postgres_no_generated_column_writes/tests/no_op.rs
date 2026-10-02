use super::*;

#[test]
fn if_not_exists_preserves_existing_state_and_accepts_missing_definitions() {
    let root = unit_fixture("if-not-exists");
    let findings = check_with_files(
        &root,
        &config_with_options("triggerMaintainedColumns: [updated_at]"),
        &[root.join("schema.sql"), root.join("writes.sql")],
    )
    .unwrap();
    assert_eq!(findings.len(), 5, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [1, 2, 3, 3, 4]
    );
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.message.contains("trigger-maintained column"))
            .count(),
        2
    );
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.message.contains("generated column"))
            .count(),
        3
    );
}
