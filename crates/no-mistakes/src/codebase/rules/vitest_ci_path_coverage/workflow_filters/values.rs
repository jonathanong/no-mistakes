use super::{workflow_finding, RuleFinding};
use serde_yaml::Value;
use std::path::Path;

#[cfg(test)]
mod tests;

pub(super) fn parse_filters_value(
    root: &Path,
    rel: &str,
    job_id: &str,
    step_id: &str,
    raw_filters: &str,
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
    let source = match std::fs::read_to_string(root.join(path)) {
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

pub(super) fn filter_patterns(value: &Value) -> Vec<String> {
    match value {
        Value::Sequence(items) => items.iter().flat_map(filter_patterns).collect(),
        Value::String(pattern) => vec![pattern.clone()],
        Value::Mapping(map) => {
            if let Some(paths) = map.get("paths") {
                return filter_patterns(paths);
            }
            map.values().flat_map(filter_patterns).collect()
        }
        _ => Vec::new(),
    }
}
