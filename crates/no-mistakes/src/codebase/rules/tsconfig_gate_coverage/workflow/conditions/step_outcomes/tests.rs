use super::*;

#[test]
fn tolerated_failures_keep_distinct_outcome_and_conclusion() {
    let step: Value = serde_yaml::from_str("id: test").unwrap();
    let mut outcomes = StepOutcomes::default();
    outcomes.record_with_conclusion(
        &step,
        StaticValue::String("failure".into()),
        StaticValue::String("success".into()),
    );

    assert_eq!(
        outcomes.value("test"),
        StaticValue::String("failure".into())
    );
    assert_eq!(
        outcomes.conclusion("test"),
        StaticValue::String("success".into())
    );
}
