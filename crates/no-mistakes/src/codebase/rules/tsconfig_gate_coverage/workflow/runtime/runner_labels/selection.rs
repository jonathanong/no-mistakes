use serde_yaml::{Mapping, Value};

use super::super::super::conditions::{
    resolve_static_interpolations, EnvironmentState, InputState,
};

#[derive(Debug, Eq, PartialEq)]
pub(super) struct StaticRunnerSelection {
    pub(super) group: Option<String>,
    pub(super) labels: Vec<String>,
}

pub(super) fn static_runner_selection(
    job: Option<&Mapping>,
    inputs: &InputState,
) -> Option<StaticRunnerSelection> {
    match job?.get("runs-on") {
        Some(Value::String(label)) => Some(StaticRunnerSelection {
            group: None,
            labels: vec![resolved_static_runner_label(label, inputs)?],
        }),
        Some(Value::Sequence(labels)) if !labels.is_empty() => labels
            .iter()
            .map(|label| {
                label
                    .as_str()
                    .and_then(|label| resolved_static_runner_label(label, inputs))
            })
            .collect::<Option<Vec<_>>>()
            .map(|labels| StaticRunnerSelection {
                group: None,
                labels,
            }),
        Some(Value::Mapping(selection)) => runner_selection(selection, inputs),
        _ => None,
    }
}

fn runner_selection(selection: &Mapping, inputs: &InputState) -> Option<StaticRunnerSelection> {
    let group = match selection.get("group").and_then(Value::as_str) {
        Some(group) => Some(resolved_static_runner_label(group, inputs)?),
        None => None,
    };
    let labels = match selection.get("labels") {
        Some(Value::String(label)) => vec![resolved_static_runner_label(label, inputs)?],
        Some(Value::Sequence(labels)) if !labels.is_empty() => labels
            .iter()
            .map(|label| {
                label
                    .as_str()
                    .and_then(|label| resolved_static_runner_label(label, inputs))
            })
            .collect::<Option<Vec<_>>>()?,
        None if group.is_some() => Vec::new(),
        None | Some(_) => return None,
    };
    Some(StaticRunnerSelection { group, labels })
}

/// Resolve a complete, context-free literal expression before interpreting its
/// runner label. Interpolated or context-dependent labels cannot prove a job
/// is schedulable on a particular runner.
fn resolved_static_runner_label(label: &str, inputs: &InputState) -> Option<String> {
    let label = label.trim();
    if label.is_empty() {
        return None;
    }
    if !label.contains("${{") {
        return Some(label.to_string());
    }
    resolve_static_interpolations(label, inputs, &EnvironmentState::default())
        .filter(|label| !label.trim().is_empty())
}

#[cfg(test)]
mod tests;
