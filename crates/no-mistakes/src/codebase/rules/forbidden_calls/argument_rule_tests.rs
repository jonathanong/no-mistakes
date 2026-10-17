use crate::codebase::rules::run_check;

fn check(config: &str) -> Vec<crate::codebase::rules::RuleFinding> {
    let source = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    let saved = crate::test_support::materialize_saved_fixture(&source);
    run_check(saved.path(), Some(&saved.path().join(config)), None).unwrap()
}

#[test]
fn literal_arguments_keep_binding_aware_target_and_application_identity() {
    let findings = check(".no-mistakes.yml");
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        vec![14, 15]
    );
    assert!(findings
        .iter()
        .all(|f| f.message.contains("no-live-analyzer-from-tests")));
    assert!(findings
        .iter()
        .all(|f| f.target.as_deref() == Some("module export `node:child_process#execFile`")));
}

#[test]
fn unsafe_aliases_obey_both_explicit_unknown_policies() {
    for (name, unknown, known) in [
        ("escape", 1, 0),
        ("destructuring", 1, 0),
        ("repeated", 3, 1),
    ] {
        let findings = check(&format!("{name}-finding.yml"));
        assert_eq!(findings.len(), unknown + known, "{name}: {findings:?}");
        assert_eq!(
            findings
                .iter()
                .filter(|f| f
                    .target
                    .as_deref()
                    .is_some_and(|v| v.starts_with("unknown arguments")))
                .count(),
            unknown
        );
        let findings = check(&format!("{name}-ignore.yml"));
        assert_eq!(findings.len(), known, "{name}: {findings:?}");
    }
}

#[test]
fn documented_selector_mapping_is_executable_configuration() {
    let findings = check("documented.yml");
    assert_eq!(
        findings.iter().map(|f| f.line).collect::<Vec<_>>(),
        vec![14, 15]
    );
}

#[test]
fn omitted_and_null_conditions_keep_target_only_demand_and_findings() {
    let root = crate::test_support::rule_fixture_root("forbidden-calls/argument-predicates");
    for name in ["target-only.yml", "null-arguments.yml"] {
        let config: crate::config::v2::NoMistakesConfig =
            serde_yaml::from_str(&std::fs::read_to_string(root.join(name)).unwrap()).unwrap();
        assert!(!super::argument_facts_required(&config));
    }
    let config: crate::config::v2::NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    assert!(super::argument_facts_required(&config));
    assert_eq!(check("target-only.yml"), check("null-arguments.yml"));
}
