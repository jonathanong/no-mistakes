use super::*;

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
        "runs-on: [self-hosted, linux]",
        "runs-on: [self-hosted, macOS-14]",
    ] {
        let job: Value = serde_yaml::from_str(yaml).unwrap();
        assert!(!runs_on_can_default_to_windows(&job), "{yaml}");
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
        ("runs-on: windows-latest", ContainerRunnerSupport::NonLinux),
        ("runs-on: macos-14", ContainerRunnerSupport::NonLinux),
        (
            "runs-on: [ubuntu-latest, windows-latest]",
            ContainerRunnerSupport::NonLinux,
        ),
        ("runs-on: custom-runner", ContainerRunnerSupport::Unknown),
        (
            "runs-on: '${{ matrix.runner }}'",
            ContainerRunnerSupport::Unknown,
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
