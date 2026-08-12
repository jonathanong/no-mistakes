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

#[test]
fn output_merges_discard_failed_scans_and_mark_divergent_values_unknown() {
    let value = StaticValue::String("one".to_string());
    let mut reusable = Some(BTreeMap::from([("value".to_string(), value.clone())]));
    merge_reusable_outputs(
        &mut reusable,
        &ActivationScan {
            projects: BTreeSet::new(),
            failed: true,
            indeterminate: false,
            outputs: BTreeMap::from([("value".to_string(), value.clone())]),
            job_outputs: BTreeMap::new(),
        },
    );
    assert_eq!(reusable, Some(BTreeMap::new()));

    let mut reusable = Some(BTreeMap::from([("value".to_string(), value.clone())]));
    merge_reusable_outputs(
        &mut reusable,
        &ActivationScan {
            projects: BTreeSet::new(),
            failed: false,
            indeterminate: false,
            outputs: BTreeMap::from([(
                "value".to_string(),
                StaticValue::String("two".to_string()),
            )]),
            job_outputs: BTreeMap::new(),
        },
    );
    assert_eq!(
        reusable,
        Some(BTreeMap::from([(
            "value".to_string(),
            StaticValue::Unknown
        )]))
    );

    let mut ordinary = Some(BTreeMap::from([("value".to_string(), value)]));
    merge_step_job_outputs(
        &mut ordinary,
        BTreeMap::from([("value".to_string(), StaticValue::String("two".to_string()))]),
    );
    assert_eq!(
        ordinary,
        Some(BTreeMap::from([(
            "value".to_string(),
            StaticValue::Unknown
        )]))
    );
}
