use super::super::globs::{compile_patterns, CompiledGlob};
use serde_yaml::Value;

pub(super) fn workflow_path_filters(value: &Value) -> Vec<Vec<CompiledGlob>> {
    workflow_path_patterns(value)
        .into_iter()
        .filter_map(|patterns| compile_patterns(&patterns).ok())
        .collect()
}

fn workflow_path_patterns(value: &Value) -> Vec<Vec<String>> {
    let Some(on) = value.get("on") else {
        return Vec::new();
    };
    match on {
        Value::Mapping(events) => events
            .iter()
            .filter_map(|(_, event)| event_paths(event))
            .collect(),
        _ => Vec::new(),
    }
}

fn event_paths(event: &Value) -> Option<Vec<String>> {
    event
        .get("paths")
        .and_then(Value::as_sequence)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|patterns| !patterns.is_empty())
}
