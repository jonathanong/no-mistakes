use super::*;

#[test]
fn missing_runner_cannot_imply_a_windows_default() {
    assert!(!runs_on_can_default_to_windows(&Value::Null));
}

#[test]
fn workflow_requires_at_least_one_file_trigger() {
    for yaml in [
        "jobs: {}",
        "on: []",
        "on: {}",
        "on: workflow_dispatch",
        "on: workflow_call",
        "on: schedule",
        "on:\n  push:\n    tags: ['v*']",
    ] {
        let workflow: Value = serde_yaml::from_str(yaml).unwrap();
        assert!(!has_file_trigger(&workflow, "ci.yml"), "{yaml}");
    }
    for yaml in ["on: push", "on: [push]", "on:\n  pull_request:"] {
        let workflow: Value = serde_yaml::from_str(yaml).unwrap();
        assert!(has_file_trigger(&workflow, "ci.yml"), "{yaml}");
    }
}
