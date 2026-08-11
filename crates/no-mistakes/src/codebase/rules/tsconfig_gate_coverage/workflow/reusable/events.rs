use super::model::GithubEventContext;
use serde_yaml::Value;

const DEFAULT_PULL_REQUEST_ACTIVITY_TYPES: &[&str] = &["opened", "synchronize", "reopened"];

/// Direct source changes occur only for the `synchronize` pull-request action.
/// Keep that action paired with its triggering event so `github.event.action`
/// conditions are evaluated against the same activation that ran the workflow.
pub(super) fn source_change_event_contexts(
    workflow: &Value,
    event: &str,
) -> Vec<GithubEventContext> {
    if !matches!(event, "pull_request" | "pull_request_target") {
        return vec![GithubEventContext::without_action(event)];
    }
    pull_request_activity_types(workflow, event)
        .filter(|action| *action == "synchronize")
        .map(|action| GithubEventContext::with_action(event, action))
        .collect()
}

fn pull_request_activity_types<'a>(
    workflow: &'a Value,
    event: &'a str,
) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    let configured = workflow
        .get("on")
        .and_then(Value::as_mapping)
        .and_then(|events| events.get(event))
        .and_then(|config| config.get("types"))
        .and_then(Value::as_sequence);
    match configured {
        Some(types) => Box::new(types.iter().filter_map(Value::as_str)),
        None => Box::new(DEFAULT_PULL_REQUEST_ACTIVITY_TYPES.iter().copied()),
    }
}

#[cfg(test)]
mod tests;
