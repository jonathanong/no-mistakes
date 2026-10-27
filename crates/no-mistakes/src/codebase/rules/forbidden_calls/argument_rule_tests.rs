#[test]
fn omitted_and_null_conditions_do_not_require_argument_facts() {
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
}
