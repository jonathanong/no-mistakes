use super::*;

#[test]
fn partial_mappings_keep_explicit_paths_without_assuming_inherited_values() {
    let value = serde_yaml::from_str(include_str!(
        "../../../../../../../test-cases/structured-value/merge-keys/partial-paths.yml"
    ))
    .unwrap();

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
