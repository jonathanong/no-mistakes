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
    let self_hosted = labels
        .iter()
        .any(|label| label.eq_ignore_ascii_case("self-hosted"));
    if self_hosted {
        return !labels
            .iter()
            .any(|label| is_explicit_self_hosted_linux_label(label));
    }
    if labels.iter().any(|label| is_windows_runner_label(label)) {
        return true;
    }
    false
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
    if labels
        .iter()
        .any(|label| label.eq_ignore_ascii_case("self-hosted"))
    {
        if labels
            .iter()
            .any(|label| is_explicit_self_hosted_linux_label(label))
        {
            ContainerRunnerSupport::Linux
        } else {
            ContainerRunnerSupport::Unknown
        }
    } else if labels
        .iter()
        .any(|label| is_windows_runner_label(label) || is_macos_runner_label(label))
    {
        ContainerRunnerSupport::NonLinux
    } else if labels.iter().any(|label| is_linux_runner_label(label)) {
        ContainerRunnerSupport::Linux
    } else {
        ContainerRunnerSupport::Unknown
    }
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

/// `ubuntu-*` and `linux-*` are useful GitHub-hosted labels, but arbitrary
/// self-hosted labels only state an OS when the exact `linux` label is present.
fn is_explicit_self_hosted_linux_label(label: &str) -> bool {
    label.trim().eq_ignore_ascii_case("linux")
}

fn is_windows_runner_label(label: &str) -> bool {
    label_prefix_matches(label, "windows")
}

fn is_linux_runner_label(label: &str) -> bool {
    ["linux", "ubuntu"]
        .iter()
        .any(|os| label_prefix_matches(label, os))
}

fn is_macos_runner_label(label: &str) -> bool {
    label_prefix_matches(label, "macos")
}

fn label_prefix_matches(label: &str, os: &str) -> bool {
    let label = label.trim();
    label.eq_ignore_ascii_case(os)
        || label
            .get(..os.len() + 1)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(&format!("{os}-")))
}
