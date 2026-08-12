use super::{
    container_runner_support as container_runner_support_for_inputs,
    has_static_runnable_runs_on as has_static_runnable_runs_on_for_inputs,
    runs_on_can_default_to_windows as runs_on_can_default_to_windows_for_inputs,
    runs_on_has_statically_invalid_value as runs_on_has_statically_invalid_value_for_inputs,
    ContainerRunnerSupport,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    inputs_with_matrix_values, InputState, MatrixState, StaticValue,
};
use serde_yaml::Value;
use std::collections::BTreeMap;

fn has_static_runnable_runs_on(job: &Value) -> bool {
    has_static_runnable_runs_on_for_inputs(job, &InputState::new())
}

#[test]
fn known_non_stringable_runs_on_expressions_are_statically_invalid() {
    let inputs = InputState::from([("runner".to_string(), StaticValue::Sequence(Vec::new()))]);
    let job: Value = serde_yaml::from_str("runs-on: '${{ inputs.runner }}'").unwrap();

    assert!(runs_on_has_statically_invalid_value_for_inputs(
        &job, &inputs
    ));
}

fn runs_on_can_default_to_windows(job: &Value) -> bool {
    runs_on_can_default_to_windows_for_inputs(job, &InputState::new())
}

fn container_runner_support(job: &serde_yaml::Mapping) -> ContainerRunnerSupport {
    container_runner_support_for_inputs(job, &InputState::new())
}

#[test]
fn pipefail_requires_an_explicit_bash_shell() {
    assert!(!super::shell_pipefail_enforced(None));
    assert!(super::shell_pipefail_enforced(Some("bash")));
    assert!(super::shell_pipefail_enforced(Some(
        "bash --noprofile --norc -eo pipefail {0}"
    )));
    assert!(!super::shell_pipefail_enforced(Some("bash -e {0}")));
}

#[test]
fn missing_runner_cannot_imply_a_windows_default() {
    assert!(!runs_on_can_default_to_windows(&Value::Null));
}

#[test]
fn bare_self_hosted_runner_keeps_the_implicit_shell_indeterminate() {
    for yaml in ["runs-on: self-hosted", "runs-on: [self-hosted]"] {
        let job: Value = serde_yaml::from_str(yaml).unwrap();
        assert!(runs_on_can_default_to_windows(&job), "{yaml}");
    }
    for yaml in [
        "runs-on: ubuntu-latest",
        "runs-on: macos-14",
        "runs-on: [self-hosted, linux]",
        "runs-on: [self-hosted, linux, x64, gpu]",
        // GitHub documents these exact self-hosted OS labels.
        "runs-on: [self-hosted, macOS]",
    ] {
        let job: Value = serde_yaml::from_str(yaml).unwrap();
        assert!(!runs_on_can_default_to_windows(&job), "{yaml}");
    }
    for yaml in [
        "runs-on: [self-hosted, ubuntu-custom]",
        "runs-on: [self-hosted, linux-custom]",
        "runs-on: [self-hosted, macOS-14]",
        "runs-on: [self-hosted, macos-custom]",
        "runs-on: [self-hosted, windows-custom]",
        "runs-on: custom-runner",
        "runs-on: ubuntu-custom",
        "runs-on: linux-custom",
        "runs-on: macos-custom",
        "runs-on: windows-custom",
    ] {
        let job: Value = serde_yaml::from_str(yaml).unwrap();
        assert!(runs_on_can_default_to_windows(&job), "{yaml}");
    }
}

#[test]
fn container_runner_support_requires_unambiguous_linux_labels() {
    for (yaml, expected) in [
        ("runs-on: ubuntu-latest", ContainerRunnerSupport::Linux),
        (
            "runs-on: [self-hosted, linux]",
            ContainerRunnerSupport::Linux,
        ),
        (
            "runs-on: [self-hosted, linux, x64, gpu]",
            ContainerRunnerSupport::Linux,
        ),
        ("runs-on: windows-latest", ContainerRunnerSupport::NonLinux),
        ("runs-on: macos-14", ContainerRunnerSupport::NonLinux),
        (
            "runs-on: [ubuntu-latest, windows-latest]",
            ContainerRunnerSupport::Unknown,
        ),
        ("runs-on: custom-runner", ContainerRunnerSupport::Unknown),
        (
            "runs-on: '${{ matrix.runner }}'",
            ContainerRunnerSupport::Unknown,
        ),
        (
            "runs-on: [self-hosted, ubuntu-custom]",
            ContainerRunnerSupport::Unknown,
        ),
        (
            "runs-on: [self-hosted, linux-custom]",
            ContainerRunnerSupport::Unknown,
        ),
        (
            "runs-on: [self-hosted, macos-custom]",
            ContainerRunnerSupport::Unknown,
        ),
        (
            "runs-on: [self-hosted, windows-custom]",
            ContainerRunnerSupport::Unknown,
        ),
        (
            "runs-on: [self-hosted, linux]",
            ContainerRunnerSupport::Linux,
        ),
        (
            "runs-on: [self-hosted, macOS]",
            ContainerRunnerSupport::NonLinux,
        ),
        (
            "runs-on: [self-hosted, windows]",
            ContainerRunnerSupport::NonLinux,
        ),
        ("runs-on: ubuntu-custom", ContainerRunnerSupport::Unknown),
        ("runs-on: macos-custom", ContainerRunnerSupport::Unknown),
        ("runs-on: windows-custom", ContainerRunnerSupport::Unknown),
        (
            "runs-on: \"${{ 'ubuntu-latest' }}\"",
            ContainerRunnerSupport::Linux,
        ),
    ] {
        let job: Value = serde_yaml::from_str(yaml).unwrap();
        assert!(
            job.as_mapping()
                .is_some_and(|job| container_runner_support(job) == expected),
            "{yaml}"
        );
    }
}

#[test]
fn documented_hosted_runner_labels_have_known_platforms() {
    let ubuntu: Value = serde_yaml::from_str("runs-on: ubuntu-26.04-arm").unwrap();
    assert!(!runs_on_can_default_to_windows(&ubuntu));

    for yaml in [
        "runs-on: macos-26-intel",
        "runs-on: macos-latest-large",
        "runs-on: macos-14-large",
        "runs-on: macos-15-large",
        "runs-on: macos-26-large",
        "runs-on: macos-latest-xlarge",
        "runs-on: macos-14-xlarge",
        "runs-on: macos-15-xlarge",
        "runs-on: macos-26-xlarge",
        "runs-on: xcode-27-xlarge",
    ] {
        let job: Value = serde_yaml::from_str(yaml).unwrap();
        assert!(!runs_on_can_default_to_windows(&job), "{yaml}");
        assert!(
            matches!(
                container_runner_support(job.as_mapping().unwrap()),
                ContainerRunnerSupport::NonLinux
            ),
            "{yaml}"
        );
    }
    for yaml in ["runs-on: macos-13-large", "runs-on: macos-13-xlarge"] {
        let job: Value = serde_yaml::from_str(yaml).unwrap();
        assert!(runs_on_can_default_to_windows(&job), "{yaml}");
    }
    let job: Value = serde_yaml::from_str("runs-on: windows-2025-vs2026").unwrap();
    assert!(runs_on_can_default_to_windows(&job));
}

#[test]
fn constant_runner_expressions_are_static_but_dynamic_expressions_are_not() {
    for yaml in [
        "runs-on: \"${{ 'ubuntu-latest' }}\"",
        "runs-on: [self-hosted, \"${{ 'linux' }}\"]",
    ] {
        let job: Value = serde_yaml::from_str(yaml).unwrap();
        assert!(has_static_runnable_runs_on(&job), "{yaml}");
        assert!(!runs_on_can_default_to_windows(&job), "{yaml}");
    }
    let dynamic: Value = serde_yaml::from_str("runs-on: '${{ matrix.runner }}'").unwrap();
    assert!(!has_static_runnable_runs_on(&dynamic));
}

#[test]
fn conflicting_and_malformed_runner_labels_remain_indeterminate() {
    let conflicting: Value =
        serde_yaml::from_str("runs-on: [self-hosted, linux, windows]").unwrap();
    assert!(runs_on_can_default_to_windows(&conflicting));
    assert!(matches!(
        container_runner_support(conflicting.as_mapping().unwrap()),
        ContainerRunnerSupport::Unknown
    ));

    for yaml in ["runs-on: []", "runs-on: 42", "runs-on: '   '"] {
        let job: Value = serde_yaml::from_str(yaml).unwrap();
        assert!(!has_static_runnable_runs_on(&job), "{yaml}");
        assert!(!runs_on_can_default_to_windows(&job), "{yaml}");
    }
}

#[test]
fn incompatible_hosted_runner_labels_are_not_runnable() {
    let job: Value = serde_yaml::from_str("runs-on: [ubuntu-latest, windows-latest]").unwrap();
    assert!(!has_static_runnable_runs_on(&job));
}

#[test]
fn runner_selection_resolves_matrix_labels_and_group_mappings() {
    let inputs = inputs_with_matrix_values(
        &InputState::new(),
        &BTreeMap::from([("os".to_string(), Value::String("ubuntu-latest".to_string()))]),
        MatrixState::Static,
    );
    let matrix: Value = serde_yaml::from_str("runs-on: '${{ matrix.os }}'").unwrap();
    assert!(has_static_runnable_runs_on_for_inputs(&matrix, &inputs));
    assert!(!runs_on_can_default_to_windows_for_inputs(&matrix, &inputs));
    assert!(matches!(
        container_runner_support_for_inputs(matrix.as_mapping().unwrap(), &inputs),
        ContainerRunnerSupport::Linux
    ));

    for yaml in [
        "runs-on: {group: ubuntu-runners}",
        "runs-on: {group: ubuntu-runners, labels: ubuntu-latest}",
        "runs-on: {labels: [ubuntu-latest]}",
    ] {
        let job: Value = serde_yaml::from_str(yaml).unwrap();
        assert!(has_static_runnable_runs_on(&job), "{yaml}");
    }

    let group_only: Value = serde_yaml::from_str("runs-on: {group: ubuntu-latest}").unwrap();
    assert!(runs_on_can_default_to_windows(&group_only));
    assert!(matches!(
        container_runner_support(group_only.as_mapping().unwrap()),
        ContainerRunnerSupport::Unknown
    ));

    let group_hosted_looking: Value =
        serde_yaml::from_str("runs-on: {group: custom, labels: ubuntu-latest}").unwrap();
    assert!(runs_on_can_default_to_windows(&group_hosted_looking));
    assert!(matches!(
        container_runner_support(group_hosted_looking.as_mapping().unwrap()),
        ContainerRunnerSupport::Unknown
    ));

    let group_linux: Value =
        serde_yaml::from_str("runs-on: {group: custom, labels: linux}").unwrap();
    assert!(!runs_on_can_default_to_windows(&group_linux));
    assert!(matches!(
        container_runner_support(group_linux.as_mapping().unwrap()),
        ContainerRunnerSupport::Linux
    ));
}
