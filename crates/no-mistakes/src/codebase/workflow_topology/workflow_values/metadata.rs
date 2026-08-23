use super::super::model;
use super::super::value_primitives;
use serde_yaml::Value;

pub fn parse_environment(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(name) => Some(name.clone()),
        Value::Mapping(mapping) => mapping
            .get(Value::String("name".to_string()))
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    }
}

pub fn parse_timeout_minutes(value: Option<&Value>) -> Option<serde_json::Number> {
    let Value::Number(number) = value? else {
        return None;
    };
    value_primitives::yaml_number_to_json(number)
}

pub fn parse_runs_on(value: Option<&Value>) -> Option<model::WorkflowRunsOn> {
    match value? {
        Value::String(label) => Some(model::WorkflowRunsOn::Label(label.clone())),
        Value::Sequence(items) => Some(model::WorkflowRunsOn::Labels(
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect(),
        )),
        Value::Mapping(mapping) => Some(model::WorkflowRunsOn::Group(model::WorkflowRunsOnGroup {
            group: mapping
                .get(Value::String("group".to_string()))
                .and_then(Value::as_str)?
                .to_string(),
            labels: parse_runs_on_labels(mapping.get(Value::String("labels".to_string()))),
        })),
        _ => None,
    }
}

fn parse_runs_on_labels(value: Option<&Value>) -> Option<model::WorkflowRunsOnLabels> {
    match value? {
        Value::String(label) => Some(model::WorkflowRunsOnLabels::Label(label.clone())),
        Value::Sequence(items) => Some(model::WorkflowRunsOnLabels::Labels(
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect(),
        )),
        _ => None,
    }
}
