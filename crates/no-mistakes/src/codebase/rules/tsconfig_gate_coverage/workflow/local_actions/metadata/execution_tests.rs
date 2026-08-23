use super::*;

fn composite_step(yaml: &str) -> bool {
    composite_step_valid(
        &serde_yaml::from_str(yaml).unwrap(),
        &BTreeMap::new(),
        &BTreeSet::new(),
        &mut BTreeSet::new(),
        &mut BTreeMap::new(),
    )
}

#[test]
fn composite_execution_rejects_malformed_steps_shells_and_directories() {
    assert!(!composite_step("invalid"));
    assert!(!composite_step("name: missing-execution"));
    assert!(!composite_step("if: always()\nrun: echo missing-shell"));
    assert!(composite_run_has_static_failure("echo ok", None));
    assert!(composite_run_has_static_failure("echo ok", Some("${{")));

    let tracked = BTreeSet::new();
    for yaml in [
        "working-directory: []",
        "working-directory: '${{ fromJSON(\"not-json\") }}'",
    ] {
        let step: Value = serde_yaml::from_str(yaml).unwrap();
        assert!(!composite_step_working_directory_valid(
            step.as_mapping().unwrap(),
            &tracked
        ));
    }
}
