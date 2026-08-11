use super::model::GithubEventContext;
use serde_yaml::Value;
use std::collections::BTreeSet;

const DEFAULT_PULL_REQUEST_ACTIVITY_TYPES: &[&str] = &["opened", "synchronize", "reopened"];

/// Direct source changes occur only for the `synchronize` pull-request action.
/// Keep that action paired with its triggering event so `github.event.action`
/// conditions are evaluated against the same activation that ran the workflow.
pub(super) fn source_change_event_contexts(
    workflow: &Value,
    event: &str,
) -> Vec<GithubEventContext> {
    if !source_change_branch_can_match(workflow, event) {
        return Vec::new();
    }
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

/// Return false only for branch filters GitHub guarantees reject every branch.
///
/// A `branches-ignore: ["**"]` filter excludes every branch. In an ordered
/// `branches` filter, `!**` resets every branch to excluded. A later positive
/// pattern can re-include branches, unless a later identical negative pattern
/// excludes that exact glob again. Other glob overlaps may leave some branch
/// eligible, so the coverage scan keeps them.
fn source_change_branch_can_match(workflow: &Value, event: &str) -> bool {
    let Some(config) = workflow
        .get("on")
        .and_then(Value::as_mapping)
        .and_then(|events| events.get(event))
    else {
        return true;
    };

    let branches_ignore = configured_patterns(config, "branches-ignore");
    if branches_ignore.contains(&"**") {
        return false;
    }

    let branches = configured_patterns(config, "branches");
    let Some(last_universal_exclusion) = branches.iter().rposition(|pattern| *pattern == "!**")
    else {
        return true;
    };
    let mut reintroduced_patterns = BTreeSet::new();
    for pattern in &branches[last_universal_exclusion + 1..] {
        match pattern.strip_prefix('!') {
            Some(excluded) => {
                reintroduced_patterns.remove(excluded);
            }
            None => {
                reintroduced_patterns.insert(*pattern);
            }
        }
    }
    !reintroduced_patterns.is_empty()
}

fn configured_patterns<'a>(config: &'a Value, key: &str) -> Vec<&'a str> {
    match config.get(key) {
        Some(Value::String(pattern)) => vec![pattern],
        Some(Value::Sequence(patterns)) => patterns.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests;
