use super::super::super::conditions::{
    complete_expression_static_value, resolve_static_interpolations, EnvironmentState, InputState,
    StaticValue,
};
use serde_yaml::Value;

pub(in super::super::super) fn runs_on_has_statically_invalid_value(
    job: &Value,
    inputs: &InputState,
) -> bool {
    runner_labels(job.get("runs-on")).any(|label| runner_label_is_statically_invalid(label, inputs))
}

fn runner_labels(value: Option<&Value>) -> impl Iterator<Item = &str> {
    value.into_iter().flat_map(|value| match value {
        Value::String(label) => vec![label.as_str()],
        Value::Sequence(labels) => labels.iter().filter_map(Value::as_str).collect(),
        Value::Mapping(selection) => ["group", "labels"]
            .into_iter()
            .flat_map(|field| match selection.get(field) {
                Some(Value::String(label)) => vec![label.as_str()],
                Some(Value::Sequence(labels)) => labels.iter().filter_map(Value::as_str).collect(),
                _ => Vec::new(),
            })
            .collect(),
        _ => Vec::new(),
    })
}

fn runner_label_is_statically_invalid(label: &str, inputs: &InputState) -> bool {
    let label = label.trim();
    if label.is_empty() {
        return true;
    }
    if !label.contains("${{") {
        return false;
    }
    if let Some(resolved) =
        resolve_static_interpolations(label, inputs, &EnvironmentState::default())
    {
        return resolved.trim().is_empty();
    }
    matches!(
        complete_expression_static_value(label, inputs),
        Some(
            StaticValue::Sequence(_)
                | StaticValue::Mapping
                | StaticValue::MatrixMapping(_)
                | StaticValue::NonStringable
                | StaticValue::Invalid
        )
    )
}
