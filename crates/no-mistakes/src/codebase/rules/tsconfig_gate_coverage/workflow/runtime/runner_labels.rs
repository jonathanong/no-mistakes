use super::super::complete_literal_expression_value;
use serde_yaml::{Mapping, Value};

/// A CI job cannot provide a typecheck gate unless Actions can schedule it on
/// a statically known runner. Reusable-workflow jobs use `uses:` rather than
/// `steps:` and are excluded separately by the step requirement.
pub(in super::super) fn has_static_runnable_runs_on(job: &Value) -> bool {
    static_runner_labels(job.as_mapping()).is_some()
}

/// An unspecified Actions shell is PowerShell on Windows. Only reject this
/// known incompatible default; an explicit supported `bash`/`sh` override is
/// still safe to analyze on the same runner.
pub(in super::super) fn runs_on_can_default_to_windows(job: &Value) -> bool {
    let Some(labels) = static_runner_labels(job.as_mapping()) else {
        return false;
    };
    matches!(
        runner_platform(&labels),
        RunnerPlatform::Windows | RunnerPlatform::Unknown
    )
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(in super::super) enum ContainerRunnerSupport {
    Linux,
    NonLinux,
    Unknown,
}

pub(in super::super) fn container_runner_support(job: &Mapping) -> ContainerRunnerSupport {
    let Some(labels) = static_runner_labels(Some(job)) else {
        return ContainerRunnerSupport::Unknown;
    };
    match runner_platform(&labels) {
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

fn static_runner_labels(job: Option<&Mapping>) -> Option<Vec<String>> {
    match job?.get("runs-on") {
        Some(Value::String(label)) => Some(vec![resolved_static_runner_label(label)?]),
        Some(Value::Sequence(labels)) if !labels.is_empty() => labels
            .iter()
            .map(|label| label.as_str().and_then(resolved_static_runner_label))
            .collect(),
        _ => None,
    }
}

/// Resolve a complete, context-free literal expression before interpreting its
/// runner label. Interpolated or context-dependent labels cannot prove a job
/// is schedulable on a particular runner.
fn resolved_static_runner_label(label: &str) -> Option<String> {
    let label = label.trim();
    if label.is_empty() {
        return None;
    }
    if !label.contains("${{") {
        return Some(label.to_string());
    }
    complete_literal_expression_value(label)
        .and_then(|value| value.as_str().map(str::to_string))
        .filter(|label| !label.trim().is_empty())
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
        "macos-14",
        "macos-15",
        "macos-15-intel",
        "macos-26",
        "macos-26-intel",
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
