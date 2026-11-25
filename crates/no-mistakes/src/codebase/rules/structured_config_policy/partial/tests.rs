use super::*;

#[test]
fn partial_mappings_keep_explicit_paths_without_assuming_inherited_values() {
    let value = fixture();

    assert!(conditions_known(&value, &[PolicyWhen { key: "enabled".into() }]));
    assert!(!conditions_known(&value, &[PolicyWhen { key: "runtime.owner".into() }]));
    assert_eq!(key(&value, "runtime.rows", true).flatten().and_then(Value::as_sequence).map(Vec::len), Some(2));

    for (selector, known) in [
        ("runtime.rows.0.owner", true),
        ("runtime.rows.1.owner", false),
        ("runtime.rows.2.owner", true),
        ("runtime.rows.[].owner", false),
        ("runtime.rows.[].missing", false),
        ("runtime.rows.[].[]", true),
        ("runtime", false),
    ] {
        let assertion = ValueAssertion { key: selector.into(), ..Default::default() };
        assert_eq!(assertion_known(&value, &assertion), known, "{selector}");
    }

    let boolean = ValueAssertion {
        key: "runtime".into(),
        kind: Some(AssertionKind::Boolean),
        ..Default::default()
    };
    assert!(assertion_known(&value, &boolean));
}


fn fixture() -> Value {
    serde_yaml::from_str(include_str!(
        "../../../../../../../test-cases/structured-value/merge-keys/partial-paths.yml"
    ))
    .unwrap()
}

#[test]
fn partial_extends_only_reports_a_known_invalid_declaration() {
    let value = fixture();
    for selector in ["absent", "runtime.owner", "enabled"] {
        let assertion = ValueAssertion {
            kind: Some(AssertionKind::AncestorOverrideSubset),
            extends_key: selector.into(),
            ..Default::default()
        };
        let finding = super::super::ancestor_override_subset::validate_partial_extends(
            &value, "partial-paths.yml", &assertion,
        );
        assert!(finding.is_none(), "{selector}: {finding:?}");
    }
}
