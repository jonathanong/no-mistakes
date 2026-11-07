use super::*;
#[test]
fn helper_lock_is_checked_at_each_caller() {
    for name in ["helper-unwritten", "helper-key-mismatch"] {
        let findings = check(&fixture(name), &config()).unwrap();
        assert_eq!(findings.len(), 1, "{name}: {findings:?}");
        let finding = &findings[0];
        assert_eq!(finding.target.as_deref(), Some(CHECKS[0]));
        assert_eq!(finding.line, 7);
        assert!(finding.message.contains("helper lockUser locks at line 3"));
    }
}
#[test]
fn caller_write_satisfies_helper_lock() {
    let findings = check(&fixture("helper-written"), &config()).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}
