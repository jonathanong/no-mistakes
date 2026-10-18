use super::*;

#[test]
fn nested_reusable_aliases_count_only_actual_caller_inventory_sources() {
    let findings = run("reusable", ".no-mistakes.yml").unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert_eq!(findings[0].line, 7);
    assert!(findings[0]
        .message
        .contains("unused declared secret UNUSED in repository"));
}

#[test]
fn undeclared_reusable_actual_is_reported_in_the_caller_without_requiring_alias_inventory() {
    let findings = run("caller-undeclared", ".no-mistakes.yml").unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, ".github/workflows/check.yml");
    assert_eq!(findings[0].line, 6);
    assert!(findings[0]
        .message
        .contains("undeclared secret MISSING_SOURCE"));
}

#[test]
fn inherited_remote_missing_and_noncallable_calls_cannot_prove_names_unused() {
    for (case, reason) in [
        ("inherit", "inherits secrets"),
        ("remote", "cannot be resolved locally"),
        ("unresolved", "cannot be resolved locally"),
        ("noncallable", "cannot be resolved locally"),
    ] {
        let findings = run(case, ".no-mistakes.yml").unwrap();
        assert_eq!(findings.len(), 1, "{case}: {findings:?}");
        assert_eq!(findings[0].file, ".github/workflows/check.yml");
        assert_eq!(findings[0].line, 4);
        assert!(findings[0]
            .message
            .starts_with("secret inventory analysis incomplete:"));
        assert!(findings[0].message.contains(reason), "{case}: {findings:?}");
        assert!(!findings
            .iter()
            .any(|finding| finding.message.starts_with("unused declared")));
    }
}

#[test]
fn recursive_reusable_calls_are_incomplete_without_a_private_call_graph() {
    let findings = run("cycle", ".no-mistakes.yml").unwrap();
    assert!(!findings.is_empty());
    assert!(findings.iter().all(|finding| {
        finding
            .message
            .starts_with("secret inventory analysis incomplete:")
    }));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("reusable call cycle")));
    assert!(findings
        .iter()
        .all(|finding| finding.file.starts_with(".github/workflows/")));
}

#[test]
fn hybrid_direct_triggers_do_not_treat_contract_aliases_as_proven_repository_reads() {
    let findings = run("hybrid", ".no-mistakes.yml").unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, ".github/workflows/check.yml");
    assert_eq!(findings[0].line, 1);
    assert!(findings[0]
        .message
        .contains("secret sources depend on the trigger"));
}

#[test]
fn reusable_references_without_contract_aliases_are_incomplete_even_if_repository_declared() {
    let findings = run("unknown-contract", ".no-mistakes.yml").unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, ".github/workflows/check.yml");
    assert_eq!(findings[0].line, 6);
    assert!(findings[0]
        .message
        .contains("reusable secret has no declared contract source"));
}

#[test]
fn malformed_reusable_bindings_remain_incomplete_at_each_authored_value() {
    let findings = run("malformed-bindings", ".no-mistakes.yml").unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [5, 8, 12, 15, 19]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.file == ".github/workflows/check.yml"));
    assert!(findings.iter().all(|finding| {
        finding
            .message
            .contains("reusable secret bindings are not a static mapping or inherit")
    }));
}
