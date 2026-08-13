use super::*;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::StaticValue;
use std::collections::BTreeMap;

#[test]
fn resolved_static_volumes_are_revalidated() {
    let value: Value = serde_yaml::from_str("['${{ matrix.volume }}:/cache']").unwrap();
    let environment = EnvironmentState::default();

    assert!(valid_for_inputs(
        Some(&value),
        &BTreeMap::from([(
            "\0matrix.volume".to_string(),
            StaticValue::String("cache".to_string()),
        )]),
        &environment,
    ));
    assert!(!valid_for_inputs(
        Some(&value),
        &BTreeMap::from([(
            "\0matrix.volume".to_string(),
            StaticValue::String("./cache".to_string()),
        )]),
        &environment,
    ));
    assert!(valid_for_inputs(
        Some(&value),
        &BTreeMap::from([("\0matrix.volume".to_string(), StaticValue::Unknown,)]),
        &environment,
    ));
    let malformed: Value = serde_yaml::from_str("['${{ }}:/cache']").unwrap();
    assert!(!valid_for_inputs(
        Some(&malformed),
        &BTreeMap::new(),
        &environment,
    ));
    let marker_collision =
        Value::Sequence(vec![Value::String(format!("{DYNAMIC_EXPRESSION}:/cache"))]);
    assert!(!valid_for_inputs(
        Some(&marker_collision),
        &BTreeMap::new(),
        &environment,
    ));
}
