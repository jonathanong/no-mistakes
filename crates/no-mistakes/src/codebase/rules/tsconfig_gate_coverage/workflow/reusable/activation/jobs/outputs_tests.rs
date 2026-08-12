use super::*;

#[test]
fn ordinary_job_outputs_keep_only_static_stringable_values() {
    let job: Value = serde_yaml::from_str(
        "outputs: {enabled: '${{ true }}', dynamic: '${{ steps.set.outputs.value }}'}",
    )
    .unwrap();

    assert_eq!(
        static_step_job_outputs(&job, &InputState::new(), &EnvironmentState::default()),
        BTreeMap::from([(
            "enabled".to_string(),
            StaticValue::String("true".to_string())
        )])
    );
}
