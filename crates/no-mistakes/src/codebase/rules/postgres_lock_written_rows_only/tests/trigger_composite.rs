use super::*;

#[test]
fn composite_assignments_fire_only_base_column_triggers() {
    let root = fixture("trigger-composite");
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-lock-written-rows-only/configs/trigger-writes.yml"
    )))
    .unwrap();
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings.len(), 3, "{findings:?}");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [11, 23, 35]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some(CHECKS[0])));
}
