use super::super::conditions::InputState;
use serde_yaml::{Mapping, Value};

mod selection;
use selection::static_runner_selection;

/// A CI job cannot provide a typecheck gate unless Actions can schedule it on
/// a statically known runner. Reusable-workflow jobs use `uses:` rather than
/// `steps:` and are excluded separately by the step requirement.
pub(in super::super) fn has_static_runnable_runs_on(job: &Value, inputs: &InputState) -> bool {
    static_runner_selection(job.as_mapping(), inputs)
        .is_some_and(|selection| selection.group.is_some() || !selection.labels.is_empty())
}

/// An unspecified Actions shell is PowerShell on Windows. Only reject this
/// known incompatible default; an explicit supported `bash`/`sh` override is
/// still safe to analyze on the same runner.
pub(in super::super) fn runs_on_can_default_to_windows(job: &Value, inputs: &InputState) -> bool {
    let Some(selection) = static_runner_selection(job.as_mapping(), inputs) else {
        return false;
    };
    matches!(
        runner_selection_platform(&selection),
        RunnerPlatform::Windows | RunnerPlatform::Unknown
    )
}

pub(in super::super) fn runner_os(job: &Value, inputs: &InputState) -> Option<&'static str> {
    let selection = static_runner_selection(job.as_mapping(), inputs)?;
    match runner_platform(&selection.labels) {
        RunnerPlatform::Linux => Some("Linux"),
        RunnerPlatform::MacOs => Some("macOS"),
        RunnerPlatform::Windows => Some("Windows"),
        RunnerPlatform::Unknown => None,
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(in super::super) enum ContainerRunnerSupport {
    Linux,
    NonLinux,
    Unknown,
}

pub(in super::super) fn container_runner_support(
    job: &Mapping,
    inputs: &InputState,
) -> ContainerRunnerSupport {
    let Some(selection) = static_runner_selection(Some(job), inputs) else {
        return ContainerRunnerSupport::Unknown;
    };
    match runner_selection_platform(&selection) {
        RunnerPlatform::Linux => ContainerRunnerSupport::Linux,
        RunnerPlatform::MacOs | RunnerPlatform::Windows => ContainerRunnerSupport::NonLinux,
        RunnerPlatform::Unknown => ContainerRunnerSupport::Unknown,
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum RunnerPlatform {
    Linux,
    MacOs,
    Windows,
    Unknown,
}

pub(in super::super) fn runner_os(job: &Value, inputs: &InputState) -> Option<&'static str> {
    let selection = static_runner_selection(job.as_mapping(), inputs)?;
    match runner_selection_platform(&selection) {
        RunnerPlatform::Linux => Some("Linux"),
        RunnerPlatform::MacOs => Some("macOS"),
        RunnerPlatform::Windows => Some("Windows"),
        RunnerPlatform::Unknown => None,
    }
}

fn runner_selection_platform(selection: &selection::StaticRunnerSelection) -> RunnerPlatform {
    if selection.group.is_some() {
        self_hosted_labels_platform(&selection.labels)
    } else {
        runner_platform(&selection.labels)
    }
}

fn runner_platform(labels: &[String]) -> RunnerPlatform {
    if labels
        .iter()
        .any(|label| label.eq_ignore_ascii_case("self-hosted"))
    {
        self_hosted_labels_platform(labels)
    } else {
        labels_platform(labels, github_hosted_runner_platform)
    }
}

fn self_hosted_labels_platform(labels: &[String]) -> RunnerPlatform {
    let mut platform = None;
    for label in labels {
        let label_platform = self_hosted_runner_platform(label);
        if label_platform == RunnerPlatform::Unknown {
            continue;
        }
        if platform.is_some_and(|known| known != label_platform) {
            return RunnerPlatform::Unknown;
        }
        platform = Some(label_platform);
    }
    platform.unwrap_or(RunnerPlatform::Unknown)
}

fn labels_platform(labels: &[String], classify: impl Fn(&str) -> RunnerPlatform) -> RunnerPlatform {
    let mut platform = None;
    for label in labels {
        if label.eq_ignore_ascii_case("self-hosted") {
            continue;
        }
        let label_platform = classify(label);
        if label_platform == RunnerPlatform::Unknown
            || platform.is_some_and(|known| known != label_platform)
        {
            return RunnerPlatform::Unknown;
        }
        platform = Some(label_platform);
    }
    platform.unwrap_or(RunnerPlatform::Unknown)
}

/// GitHub documents exact OS labels for self-hosted runners. Prefixes are
/// custom labels and cannot establish a platform.
fn self_hosted_runner_platform(label: &str) -> RunnerPlatform {
    if label.trim().eq_ignore_ascii_case("linux") {
        RunnerPlatform::Linux
    } else if label.trim().eq_ignore_ascii_case("macos") {
        RunnerPlatform::MacOs
    } else if label.trim().eq_ignore_ascii_case("windows") {
        RunnerPlatform::Windows
    } else {
        RunnerPlatform::Unknown
    }
}

/// The GitHub-hosted labels recognized here are a finite, documented set. A
/// custom label can omit `self-hosted`, so an OS-looking prefix is not proof.
fn github_hosted_runner_platform(label: &str) -> RunnerPlatform {
    let label = label.trim();
    if [
        "ubuntu-slim",
        "ubuntu-latest",
        "ubuntu-22.04",
        "ubuntu-22.04-arm",
        "ubuntu-24.04",
        "ubuntu-24.04-arm",
        "ubuntu-26.04",
        "ubuntu-26.04-arm",
    ]
    .iter()
    .any(|known| label.eq_ignore_ascii_case(known))
    {
        RunnerPlatform::Linux
    } else if [
        "macos-latest",
        "macos-latest-large",
        "macos-14",
        "macos-14-large",
        "macos-15",
        "macos-15-intel",
        "macos-15-large",
        "macos-26",
        "macos-26-intel",
        "macos-26-large",
        "macos-latest-xlarge",
        "macos-14-xlarge",
        "macos-15-xlarge",
        "macos-26-xlarge",
        "xcode-27-xlarge",
    ]
    .iter()
    .any(|known| label.eq_ignore_ascii_case(known))
    {
        RunnerPlatform::MacOs
    } else if [
        "windows-latest",
        "windows-2022",
        "windows-2025",
        "windows-2025-vs2026",
        "windows-11-arm",
        "windows-11-vs2026-arm",
    ]
    .iter()
    .any(|known| label.eq_ignore_ascii_case(known))
    {
        RunnerPlatform::Windows
    } else {
        RunnerPlatform::Unknown
    }
}
