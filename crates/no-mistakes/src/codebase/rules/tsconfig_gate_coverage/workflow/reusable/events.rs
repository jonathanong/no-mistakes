use super::model::{GithubEventContext, GithubRef};
use serde_yaml::Value;
use std::collections::BTreeSet;

const DEFAULT_PULL_REQUEST_ACTIVITY_TYPES: &[&str] = &["opened", "synchronize", "reopened"];

/// Direct source changes occur only for the `synchronize` pull-request action.
/// Keep an event's action and ref together so conditions and reusable-call
/// inputs are evaluated for the same activation.
pub(super) fn source_change_event_contexts(
    workflow: &Value,
    event: &str,
) -> Vec<GithubEventContext> {
    // The default pull_request_target checkout is the trusted base, not the
    // changed PR head. Do not credit it without an explicit head-ref model.
    if event == "pull_request_target" {
        return Vec::new();
    }
    let references = event_references(workflow, event);
    if event != "pull_request" {
        return references
            .into_iter()
            .map(|reference| GithubEventContext::with_ref(event, reference))
            .collect();
    }
    pull_request_activity_types(workflow, event)
        .filter(|action| *action == "synchronize")
        .flat_map(|action| {
            references.iter().cloned().map(move |reference| {
                let base_reference = reference.clone();
                let workflow_reference = if event == "pull_request" {
                    GithubRef::PullRequestMerge
                } else {
                    reference
                };
                GithubEventContext::with_action_and_refs(
                    event,
                    action,
                    workflow_reference,
                    base_reference,
                )
            })
        })
        .collect()
}

fn event_references(workflow: &Value, event: &str) -> Vec<GithubRef> {
    let config = workflow
        .get("on")
        .and_then(Value::as_mapping)
        .and_then(|events| events.get(event));
    match event {
        "push" => {
            let branches_configured =
                has_key(config, "branches") || has_key(config, "branches-ignore");
            let tags_configured = has_key(config, "tags") || has_key(config, "tags-ignore");
            // A tag push does not have a source branch to establish project
            // coverage. If branch filters are absent, this is tag-only (or
            // mixed unknown ref) and cannot prove a source-change gate.
            if tags_configured && !branches_configured {
                Vec::new()
            } else {
                branch_references(config)
            }
        }
        "pull_request" => branch_references(config),
        "pull_request_target" => branch_references(config),
        _ => vec![GithubRef::Unknown],
    }
}

fn branch_references(config: Option<&Value>) -> Vec<GithubRef> {
    references_for(config, "branches", "branches-ignore", "refs/heads/")
}

fn references_for(
    config: Option<&Value>,
    include_key: &str,
    ignore_key: &str,
    prefix: &str,
) -> Vec<GithubRef> {
    let Some(config) = config else {
        return vec![GithubRef::UnknownBranch];
    };
    let includes = configured_patterns(config, include_key);
    let ignores = configured_patterns(config, ignore_key);
    if includes.is_empty() {
        return if ignores.contains(&"**") {
            Vec::new()
        } else if !ignores.is_empty() {
            let excluded = ignores
                .into_iter()
                .filter(|pattern| is_exact_pattern(pattern))
                .map(|pattern| format!("{prefix}{pattern}"))
                .collect::<BTreeSet<_>>();
            if excluded.is_empty() {
                return vec![GithubRef::UnknownBranch];
            }
            vec![GithubRef::UnknownExcluding(excluded)]
        } else {
            vec![GithubRef::UnknownBranch]
        };
    }

    let last_reset = includes.iter().rposition(|pattern| *pattern == "!**");
    let patterns = last_reset.map_or(includes.as_slice(), |index| &includes[index + 1..]);
    let candidates = patterns
        .iter()
        .filter_map(|pattern| pattern.strip_prefix('!').map_or(Some(*pattern), |_| None))
        .filter(|pattern| is_exact_pattern(pattern))
        .collect::<BTreeSet<_>>();
    let mut references = candidates
        .iter()
        .filter(|candidate| selected_exact(patterns, candidate))
        .map(|candidate| GithubRef::Exact(format!("{prefix}{candidate}")))
        .collect::<Vec<_>>();
    if patterns
        .iter()
        .any(|pattern| !pattern.starts_with('!') && !is_exact_pattern(pattern))
    {
        references.push(if candidates.is_empty() {
            GithubRef::UnknownBranch
        } else {
            GithubRef::UnknownExcluding(
                candidates
                    .into_iter()
                    .map(|candidate| format!("{prefix}{candidate}"))
                    .collect(),
            )
        });
    }
    references
}

fn has_key(config: Option<&Value>, key: &str) -> bool {
    config
        .and_then(Value::as_mapping)
        .is_some_and(|config| config.contains_key(key))
}

fn configured_patterns<'a>(config: &'a Value, key: &str) -> Vec<&'a str> {
    match config.get(key) {
        Some(Value::String(pattern)) => vec![pattern],
        Some(Value::Sequence(patterns)) => patterns.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

fn is_exact_pattern(pattern: &str) -> bool {
    let pattern = pattern.strip_prefix('!').unwrap_or(pattern);
    !pattern.is_empty()
        && !pattern.contains("${{")
        && !pattern.contains(['*', '?', '+', '[', ']', '{', '}'])
}

fn selected_exact(patterns: &[&str], candidate: &str) -> bool {
    let mut selected = false;
    for pattern in patterns {
        let (negated, pattern) = pattern
            .strip_prefix('!')
            .map_or((false, *pattern), |pattern| (true, pattern));
        if pattern == candidate {
            selected = !negated;
        }
    }
    selected
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
