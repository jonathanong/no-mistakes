use super::*;

#[test]
fn replication_role_changes_disable_trigger_proof_but_keep_direct_writes() {
    let root = fixture("trigger-replication-role");
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-lock-written-rows-only/configs/trigger-writes.yml"
    )))
    .unwrap();
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [6, 13]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some(CHECKS[0])));
}
