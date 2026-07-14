use super::{workflow_finding, RuleFinding};
use serde_yaml::Value;
use std::path::Path;

#[cfg(test)]
mod tests;

pub(super) fn parse_filters_value_with_sources(
    root: &Path,
    rel: &str,
    job_id: &str,
    step_id: &str,
    raw_filters: &str,
    sources: &crate::codebase::ts_source::SourceStore,
    findings: &mut Vec<RuleFinding>,
) -> Option<Value> {
    let parsed: Value = match serde_yaml::from_str(raw_filters) {
        Ok(value) => value,
        Err(error) => {
            findings.push(workflow_finding(
                rel,
                format!(
                    "{rel}: jobs.{job_id}.steps.{step_id}.with.filters is not valid YAML: {error}"
                ),
                Some(format!("{job_id}.{step_id}")),
            ));
            return None;
        }
    };

    let Some(path) = parsed.as_str() else {
        return Some(parsed);
    };
    let source = match sources.read_path(&root.join(path)) {
        Ok(source) => source,
        Err(error) => {
            findings.push(workflow_finding(
                rel,
                format!(
                    "{rel}: jobs.{job_id}.steps.{step_id}.with.filters file `{path}` could not be read: {error}"
                ),
                Some(format!("{job_id}.{step_id}")),
            ));
            return None;
        }
    };
    match serde_yaml::from_str(&source) {
        Ok(value) => Some(value),
        Err(error) => {
            findings.push(workflow_finding(
                rel,
                format!(
                    "{rel}: jobs.{job_id}.steps.{step_id}.with.filters file `{path}` is not valid YAML: {error}"
                ),
                Some(format!("{job_id}.{step_id}")),
            ));
            None
        }
    }
}

pub(super) fn filter_predicates(value: &Value) -> Vec<Vec<String>> {
    match value {
        Value::Sequence(items) => {
            let mut predicates = Vec::new();
            for item in items {
                predicates.extend(filter_predicates(item));
            }
            predicates
        }
        Value::String(pattern) => vec![vec![pattern.clone()]],
        Value::Mapping(map) => {
            let mut predicates = Vec::new();
            for (change_types, patterns) in map {
                if change_types_cover_source_changes(change_types) {
                    predicates.extend(predicate_alternatives(patterns));
                }
            }
            predicates
        }
        _ => Vec::new(),
    }
}

fn predicate_alternatives(value: &Value) -> Vec<Vec<String>> {
    match value {
        Value::Sequence(items) => {
            let alternatives = items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect();
            vec![alternatives]
        }
        Value::String(pattern) => vec![vec![pattern.clone()]],
        Value::Mapping(map) => {
            let mut predicates = Vec::new();
            for (change_types, patterns) in map {
                if change_types_cover_source_changes(change_types) {
                    predicates.extend(predicate_alternatives(patterns));
                }
            }
            predicates
        }
        _ => Vec::new(),
    }
}

fn change_types_cover_source_changes(value: &Value) -> bool {
    let Some(raw) = value.as_str() else {
        return false;
    };
    raw.split('|')
        .any(|part| matches!(part.trim(), "added" | "modified"))
}
