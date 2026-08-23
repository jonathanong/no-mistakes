use super::{inputs_with_needs_results, needs_output_value, needs_result_value};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    InputState, StaticValue,
};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn executed_nonfailing_needs_publish_success_results() {
    let job = serde_yaml::from_str::<Value>("needs: tolerated").unwrap();
    let inputs = inputs_with_needs_results(
        &InputState::new(),
        &job,
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::from(["tolerated".to_string()]),
        &BTreeMap::new(),
    );

    assert_eq!(
        needs_result_value("tolerated", &inputs),
        StaticValue::String("success".to_string())
    );
}

#[test]
fn needs_outputs_are_normalized_and_unknown_when_absent() {
    let job = serde_yaml::from_str::<Value>("needs: reusable-call").unwrap();
    let outputs = BTreeMap::from([(
        "reusable-call".to_string(),
        BTreeMap::from([(
            "enabled".to_string(),
            StaticValue::String("false".to_string()),
        )]),
    )]);
    let inputs = inputs_with_needs_results(
        &InputState::new(),
        &job,
        &BTreeSet::new(),
        &BTreeSet::new(),
        &BTreeSet::from(["reusable-call".to_string()]),
        &outputs,
    );

    assert_eq!(
        needs_output_value("REUSABLE-CALL", "ENABLED", &inputs),
        StaticValue::String("false".to_string())
    );
    assert_eq!(
        needs_output_value("reusable-call", "missing", &inputs),
        StaticValue::Unknown
    );
}
