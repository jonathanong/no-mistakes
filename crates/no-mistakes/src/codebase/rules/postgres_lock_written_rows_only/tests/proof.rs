use super::*;

fn root(case: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-lock-written-rows-only/fixture")
        .join(case)
}

fn config(name: &str) -> NoMistakesConfig {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-lock-written-rows-only/configs")
        .join(format!("{name}.yml"));
    serde_yaml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn failing_config(name: &str) -> NoMistakesConfig {
    let mut config = config(name);
    config.rules[0].options["checks"] =
        serde_yaml::to_value(BTreeMap::from(CHECKS.map(|id| (id, "fail")))).unwrap();
    config
}

fn targets(case: &str, config: &NoMistakesConfig) -> Vec<(String, usize)> {
    check(&root(case), config)
        .unwrap()
        .into_iter()
        .map(|finding| (finding.target.unwrap_or_default(), finding.line))
        .collect()
}

fn assert_stale(finding: &RuleFinding, option: &str) {
    assert_eq!(finding.rule, "unused-allow-entry");
    assert!(finding.message.contains(RULE_ID), "{finding:?}");
    assert!(finding.message.contains(option), "{finding:?}");
}

#[test]
fn every_constant_advisory_form_including_bound_literals_is_rejected() {
    let findings = targets("constant-advisory", &failing_config("fail"));
    assert_eq!(
        findings,
        [4, 5, 6, 7, 8, 10]
            .map(|line| (CHECKS[2].to_owned(), line))
            .to_vec()
    );
}

#[test]
fn all_row_lock_strengths_and_transaction_isolation_are_checked() {
    let config = failing_config("fail");
    assert_eq!(
        targets("lock-strengths", &config),
        [4, 5, 6, 7]
            .map(|line| (CHECKS[0].to_owned(), line))
            .to_vec()
    );
    assert_eq!(
        targets("isolated-transactions", &config),
        vec![(CHECKS[0].to_owned(), 5)]
    );
}

#[test]
fn unexecuted_prepared_statement_does_not_supply_a_write() {
    assert_eq!(
        targets("prepared-write", &failing_config("fail")),
        vec![(CHECKS[0].to_owned(), 4)]
    );
}

#[test]
fn uncertain_transaction_proofs_follow_each_unanalyzable_policy() {
    for case in [
        "nested-write",
        "unknown-write",
        "cte-relation",
        "committed",
        "unresolved",
    ] {
        let findings = targets(case, &failing_config("fail"));
        assert!(
            findings.iter().any(|(target, _)| target == "unanalyzable"),
            "{case}: {findings:?}"
        );
        assert!(
            findings.iter().all(|(target, _)| target == "unanalyzable"),
            "{case}: must abstain rather than invent an unwritten-row proof: {findings:?}"
        );
        for policy in ["advise", "ignore"] {
            assert!(
                targets(case, &failing_config(policy)).is_empty(),
                "{case}: {policy} must not fail the check"
            );
        }
    }
}

#[test]
fn relation_site_and_key_exceptions_only_consume_matching_entries() {
    let findings = check(&root("row-unwritten"), &failing_config("allow")).unwrap();
    assert_eq!(findings.len(), 1);
    assert_stale(&findings[0], "allow");
    assert!(targets("row-unwritten", &failing_config("allow-call")).is_empty());
    assert_eq!(
        targets("constant-advisory", &failing_config("allow-key")),
        [5, 6, 7, 10]
            .map(|line| (CHECKS[2].to_owned(), line))
            .to_vec()
    );
    let mut config = failing_config("allow-call");
    config.rules[0].options["allow"][0]["line"] = serde_yaml::to_value(99).unwrap();
    let findings = check(&root("row-unwritten"), &config).unwrap();
    assert_eq!(findings.len(), 2);
    assert!(findings.iter().any(|finding| finding.rule == RULE_ID
        && finding.target.as_deref() == Some(CHECKS[0])
        && finding.line == 4));
    let stale = findings
        .iter()
        .find(|finding| finding.rule == "unused-allow-entry")
        .unwrap();
    assert_stale(stale, "allow");
}

#[test]
fn asserted_relation_write_does_not_excuse_a_proven_key_mismatch() {
    let mut config = failing_config("write-sets");
    config.rules[0].options["writeSets"][0]["relations"] = serde_yaml::to_value(["items"]).unwrap();
    let findings = check(&root("row-key-mismatch"), &config).unwrap();
    assert!(
        findings.iter().any(|finding| finding.rule == RULE_ID
            && finding.target.as_deref() == Some(CHECKS[0])
            && finding.line == 4),
        "{findings:?}"
    );
    let stale: Vec<_> = findings
        .iter()
        .filter(|finding| finding.rule == "unused-allow-entry")
        .collect();
    assert_eq!(stale.len(), 2);
    for finding in stale {
        assert_stale(finding, "writeSets");
    }
    let findings = check(&root("row-unwritten"), &failing_config("write-sets")).unwrap();
    assert_eq!(findings.len(), 1);
    assert_stale(&findings[0], "writeSets");
}

#[test]
fn per_check_severity_changes_failure_without_changing_lock_classification() {
    for (case, count) in [
        ("row-unwritten", 1),
        ("table-lock", 2),
        ("constant-advisory", 6),
        ("outside-transaction", 2),
    ] {
        assert_eq!(targets(case, &config("checks-fail")).len(), count, "{case}");
        assert!(
            targets(case, &config("fail")).is_empty(),
            "{case}: default advise"
        );
        assert!(
            targets(case, &config("checks-ignore")).is_empty(),
            "{case}: ignore"
        );
    }
}

#[test]
fn incomplete_sql_cannot_prove_allow_or_write_set_entries_stale() {
    for case in ["unknown-write", "unresolved"] {
        for name in ["allow", "write-sets"] {
            let findings = check(&root(case), &failing_config(name)).unwrap();
            assert!(
                findings.iter().any(|finding| finding.rule == RULE_ID
                    && finding.target.as_deref() == Some("unanalyzable")),
                "{case}/{name}: {findings:?}"
            );
            assert!(
                findings
                    .iter()
                    .all(|finding| finding.rule != "unused-allow-entry"),
                "{case}/{name}: unresolved SQL cannot prove an exception stale: {findings:?}"
            );
        }
    }
}

#[test]
fn update_from_and_delete_using_do_not_prove_matching_row_keys() {
    let findings = targets("unknown-write-predicate", &failing_config("fail"));
    assert_eq!(
        findings,
        vec![
            ("unanalyzable".to_owned(), 4),
            ("unanalyzable".to_owned(), 9)
        ]
    );
    for policy in ["advise", "ignore"] {
        assert!(targets("unknown-write-predicate", &failing_config(policy)).is_empty());
    }
}

#[test]
fn relation_identity_changes_cannot_certify_the_locked_row_is_written() {
    for (case, lines) in [("search-path", vec![4, 10]), ("temp-shadow", vec![4])] {
        let expected: Vec<_> = lines
            .into_iter()
            .map(|line| ("unanalyzable".to_owned(), line))
            .collect();
        assert_eq!(targets(case, &failing_config("fail")), expected, "{case}");
        for policy in ["advise", "ignore"] {
            assert!(
                targets(case, &failing_config(policy)).is_empty(),
                "{case}/{policy}"
            );
        }
    }
    assert!(targets("row-written", &failing_config("fail")).is_empty());
}

#[test]
fn ignored_checks_do_not_request_irrelevant_transaction_proofs() {
    for (case, name) in [
        ("typed-membership", "ignore-row-checks"),
        ("unknown-write-predicate", "ignore-written-check"),
        ("outside-transaction", "ignore-outside-check"),
        ("unresolved", "checks-ignore"),
    ] {
        assert!(targets(case, &config(name)).is_empty(), "{case}/{name}");
    }
}

#[test]
fn ignored_checks_do_not_consume_allow_entries() {
    let findings = check(&root("table-lock"), &config("ignored-allow")).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_stale(&findings[0], "allow");
}

#[test]
fn key_proofs_distinguish_unresolved_from_unwritten_rows() {
    let violations = vec![
        (CHECKS[0].to_owned(), 9),
        (CHECKS[0].to_owned(), 24),
        (CHECKS[0].to_owned(), 29),
    ];
    assert_eq!(
        targets("key-proof", &config("key-proof-ignore")),
        violations
    );
    let mut expected = violations;
    expected.push(("unanalyzable".to_owned(), 35));
    assert_eq!(targets("key-proof", &failing_config("fail")), expected);
}
