use serde_yaml::Value;
use std::collections::BTreeSet;

pub(super) fn source_change_event_eligible(workflow: &Value, event: &str) -> bool {
    if !matches!(event, "pull_request" | "pull_request_target") {
        return true;
    }
    workflow
        .get("on")
        .and_then(Value::as_mapping)
        .and_then(|events| events.get(event))
        .and_then(|config| config.get("types"))
        .is_none_or(|types| {
            types.as_sequence().is_some_and(|types| {
                types
                    .iter()
                    .any(|value| value.as_str() == Some("synchronize"))
            })
        })
}

/// Return one direct activation input per statically configured pull request
/// activity. A `types` list narrows GitHub's possible `github.event.action`
/// values, so each activity must be evaluated independently rather than
/// combining mutually exclusive condition states. Sorting/deduplicating keeps
/// the activation memo and resulting findings deterministic.
pub(super) fn direct_event_actions(workflow: &Value, event: &str) -> Vec<Option<String>> {
    if event != "pull_request" {
        return vec![None];
    }
    let actions = workflow
        .get("on")
        .and_then(Value::as_mapping)
        .and_then(|events| events.get(event))
        .and_then(|config| config.get("types"))
        .and_then(Value::as_sequence)
        .map(|types| {
            types
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        });
    match actions {
        Some(actions) if !actions.is_empty() => actions.into_iter().map(Some).collect(),
        _ => vec![None],
    }
}

#[cfg(test)]
mod tests;
