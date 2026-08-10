use serde_yaml::Value;

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

#[cfg(test)]
mod tests;
