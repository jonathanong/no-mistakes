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
#[test]
fn imported_helper_locks_use_the_caller_write_set() {
    for (name, count) in [
        ("helper-imported-unwritten", 1),
        ("helper-imported-written", 0),
    ] {
        let root = fixture(name);
        let findings = check_with_files(
            &root,
            &config(),
            &[root.join("src/query.ts"), root.join("src/helper.ts")],
        )
        .unwrap();
        assert_eq!(findings.len(), count, "{name}: {findings:?}");
        if count == 1 {
            assert_eq!(findings[0].line, 5);
            assert_eq!(findings[0].target.as_deref(), Some(CHECKS[0]));
            assert!(findings[0]
                .message
                .contains("helper acquireUser locks at line 3"));
        }
    }
}
