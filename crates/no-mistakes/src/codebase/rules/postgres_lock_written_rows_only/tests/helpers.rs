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
#[test]
fn unreachable_write_in_helper_does_not_satisfy_the_lock() {
    let findings = check(&fixture("helper-dead-write"), &config()).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some(CHECKS[0]));
    assert_eq!(findings[0].line, 9);
}
#[test]
fn helper_calls_after_caller_return_do_not_prove_writes() {
    for (name, line) in [
        ("helper-caller-exited", 7),
        ("helper-imported-caller-exited", 5),
    ] {
        let root = fixture(name);
        let mut files = vec![root.join("src/query.ts")];
        if name.contains("imported") {
            files.push(root.join("src/helper.ts"));
        }
        let findings = check_with_files(&root, &config(), &files).unwrap();
        assert!(
            findings
                .iter()
                .any(|finding| finding.line == line
                    && finding.target.as_deref() == Some("unanalyzable")),
            "{name}: {findings:?}"
        );
    }
}
#[test]
fn helper_executor_option_does_not_prove_a_write_on_the_callee_transaction() {
    let mut config = config();
    config.rules[0].options["executorOptionProperties"] =
        serde_yaml::to_value(["executor"]).unwrap();
    let findings = check(&fixture("helper-option-override"), &config).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.line == 8 && finding.target.as_deref() == Some("unanalyzable")),
        "{findings:?}"
    );
}
#[test]
fn uncalled_helper_keeps_unknown_transaction_membership_diagnostic() {
    let findings = check(&fixture("unresolved"), &config()).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.line == 7 && finding.target.as_deref() == Some("unanalyzable")),
        "{findings:?}"
    );
}
