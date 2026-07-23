//! Parse GitHub Actions workflow YAML into the typed [`Workflow`] model.

use super::model::{Job, PathFilter, PermissionLevel, PermissionSpec, Triggers, Workflow};
use anyhow::Result;
use serde_yaml::Value;
use std::collections::BTreeMap;

/// Events whose `on:` entry supports `paths` / `paths-ignore` filtering.
pub(super) const PATH_FILTERABLE_EVENTS: &[&str] = &["push", "pull_request", "pull_request_target"];

/// Parse a single workflow YAML document.
pub fn parse_workflow(yaml: &str, rel_path: &str) -> Result<Workflow> {
    let value: Value = serde_yaml::from_str(yaml)?;
    Ok(parse_workflow_value(&value, rel_path))
}

/// Build the typed impact model from a workflow YAML document parsed by the
/// request-scoped shared workflow loader.
pub fn parse_workflow_value(value: &Value, rel_path: &str) -> Workflow {
    let mut warnings = Vec::new();

    let name = value
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string);

    let (triggers, is_reusable) = parse_triggers(value.get("on"), rel_path, &mut warnings);
    let permissions = parse_permission_spec(value.get("permissions"));
    let jobs = parse_jobs(value.get("jobs"));

    Workflow {
        path: rel_path.to_string(),
        name,
        triggers,
        permissions,
        jobs,
        is_reusable,
        warnings,
    }
}

/// Parse the `on:` value in its string / list / map forms. Returns the triggers
/// and whether the workflow is reusable (`on: workflow_call`).
fn parse_triggers(
    on: Option<&Value>,
    rel_path: &str,
    warnings: &mut Vec<String>,
) -> (Triggers, bool) {
    let mut triggers = Triggers::default();
    let mut is_reusable = false;

    match on {
        Some(Value::String(event)) => add_event(&mut triggers, event, None, rel_path, warnings),
        Some(Value::Sequence(events)) => {
            for event in events {
                if let Some(name) = event.as_str() {
                    add_event(&mut triggers, name, None, rel_path, warnings);
                }
            }
        }
        Some(Value::Mapping(map)) => {
            for (key, val) in map {
                if let Some(name) = key.as_str() {
                    add_event(&mut triggers, name, Some(val), rel_path, warnings);
                }
            }
        }
        _ => {}
    }

    if triggers.events.contains_key("workflow_call")
        || triggers.other_events.iter().any(|e| e == "workflow_call")
    {
        is_reusable = true;
    }

    triggers.other_events.sort();
    triggers.other_events.dedup();
    (triggers, is_reusable)
}

fn add_event(
    triggers: &mut Triggers,
    name: &str,
    config: Option<&Value>,
    rel_path: &str,
    warnings: &mut Vec<String>,
) {
    if PATH_FILTERABLE_EVENTS.contains(&name) && !is_tag_only(config) {
        let filter = parse_path_filter(config, name, rel_path, warnings);
        triggers.events.insert(name.to_string(), filter);
    } else {
        triggers.other_events.push(name.to_string());
    }
}

/// A push filtered by `tags`/`tags-ignore` with no `branches`/`branches-ignore`
/// does not run on branch pushes, and path filters are not evaluated for tag
/// pushes — so a changed file never triggers it (even if `paths` is also set).
/// Classify it as a non-file-triggered event.
fn is_tag_only(config: Option<&Value>) -> bool {
    let Some(config) = config else {
        return false;
    };
    let has = |key: &str| config.get(key).is_some();
    (has("tags") || has("tags-ignore")) && !has("branches") && !has("branches-ignore")
}

fn parse_path_filter(
    config: Option<&Value>,
    event: &str,
    rel_path: &str,
    warnings: &mut Vec<String>,
) -> PathFilter {
    let paths = string_list(config.and_then(|c| c.get("paths")));
    let paths_ignore = string_list(config.and_then(|c| c.get("paths-ignore")));
    if !paths.is_empty() && !paths_ignore.is_empty() {
        warnings.push(format!(
            "{rel_path}: event `{event}` declares both `paths` and `paths-ignore`; \
             GitHub disallows this — using `paths`"
        ));
        return PathFilter {
            paths,
            paths_ignore: Vec::new(),
        };
    }
    PathFilter {
        paths,
        paths_ignore,
    }
}

fn string_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Sequence(seq)) => seq
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        Some(Value::String(s)) => vec![s.clone()],
        _ => Vec::new(),
    }
}

/// Parse a `permissions:` value (workflow- or job-level).
///
/// This is intentionally lenient: GitHub validates the workflow schema and
/// rejects invalid permission strings/levels, but `ci impact` only does
/// best-effort analysis, so an unknown shorthand string is treated as `Empty`
/// and an unknown level is dropped rather than erroring. Use `actionlint` for
/// schema validation.
pub(super) fn parse_permission_spec(value: Option<&Value>) -> PermissionSpec {
    match value {
        None | Some(Value::Null) => PermissionSpec::Unspecified,
        Some(Value::String(s)) => match s.as_str() {
            "read-all" => PermissionSpec::ReadAll,
            "write-all" => PermissionSpec::WriteAll,
            // `permissions: {}` parses as a mapping; a bare `none`-like string is
            // treated as an unknown single value → empty.
            _ => PermissionSpec::Empty,
        },
        Some(Value::Mapping(map)) => {
            if map.is_empty() {
                return PermissionSpec::Empty;
            }
            let mut scopes = BTreeMap::new();
            for (key, val) in map {
                if let (Some(scope), Some(level)) = (key.as_str(), val.as_str()) {
                    if let Some(level) = parse_permission_level(level) {
                        scopes.insert(scope.to_string(), level);
                    }
                }
            }
            PermissionSpec::Map(scopes)
        }
        _ => PermissionSpec::Unspecified,
    }
}

fn parse_permission_level(value: &str) -> Option<PermissionLevel> {
    match value {
        "read" => Some(PermissionLevel::Read),
        "write" => Some(PermissionLevel::Write),
        "none" => Some(PermissionLevel::None),
        _ => None,
    }
}

fn parse_jobs(jobs: Option<&Value>) -> Vec<Job> {
    let Some(map) = jobs.and_then(Value::as_mapping) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    for (key, val) in map {
        let Some(id) = key.as_str() else { continue };
        let name = val.get("name").and_then(Value::as_str).map(str::to_string);
        let permissions = parse_permission_spec(val.get("permissions"));
        let uses = val.get("uses").and_then(Value::as_str).map(str::to_string);
        result.push(Job {
            id: id.to_string(),
            name,
            permissions,
            uses,
        });
    }
    result.sort_by(|a, b| a.id.cmp(&b.id));
    result
}
