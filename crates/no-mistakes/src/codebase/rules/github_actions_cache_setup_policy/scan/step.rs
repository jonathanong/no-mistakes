use super::super::options::{normalized_path, Compiled};
use super::{RuleFinding, StepLocation};
use crate::codebase::rules::github_actions_action_input_policy::policy::family_of;
use regex::Regex;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn scan(
    step: &Value,
    location: StepLocation<'_>,
    options: &Compiled,
    findings: &mut Vec<RuleFinding>,
) {
    let Some(uses) = step.get("uses") else {
        if !step.is_mapping() {
            findings.push(location.finding(&[], "unanalyzable", "workflow step must be a mapping"));
        }
        return;
    };
    let Some(family) = uses.as_str().and_then(family_of) else {
        findings.push(location.finding(
            &["uses"],
            "unanalyzable",
            "cannot determine the action family; use a literal action reference",
        ));
        return;
    };
    if let Some(max) = options.setup.get(&family).filter(|_| location.workflow) {
        if let Some(row) = deadline(step.get("timeout-minutes"), *max, &location) {
            findings.push(row);
        }
    }
    if let Some(paths) = options.caches.get(&family) {
        cache(step.get("with"), paths, &location, findings);
    }
}

fn deadline(value: Option<&Value>, max: u64, location: &StepLocation<'_>) -> Option<RuleFinding> {
    let field = &["timeout-minutes"];
    if let Some(Value::String(text)) = value {
        if dynamic(text) {
            return Some(location.finding(
                field,
                "unanalyzable",
                "cannot verify an expression-valued setup deadline; set a literal timeout-minutes",
            ));
        }
    }
    let minutes = value.and_then(Value::as_u64);
    if minutes.is_some_and(|minutes| (1..=max).contains(&minutes)) {
        return None;
    }
    Some(location.finding(
        field,
        "setup-deadline",
        &format!(
            "configured setup action needs a literal integer timeout-minutes from 1 through {max}"
        ),
    ))
}

fn cache(
    with: Option<&Value>,
    paths: &BTreeMap<String, Regex>,
    location: &StepLocation<'_>,
    findings: &mut Vec<RuleFinding>,
) {
    let field = &["with", "path"];
    let Some(with) = with else { return };
    if !with.is_mapping() {
        findings.push(location.finding(
            &["with"],
            "unanalyzable",
            "cannot determine cache paths from a non-mapping with value",
        ));
        return;
    }
    let Some(path) = with.get("path") else { return };
    let mut entries = BTreeSet::new();
    let known = match path {
        Value::Sequence(items) => {
            let mut known = true;
            for item in items {
                known &= add_paths(item, &mut entries);
            }
            known
        }
        value => add_paths(value, &mut entries),
    };
    if !known {
        findings.push(location.finding(
            field,
            "unanalyzable",
            "cannot determine all cache paths; declare the paths literally",
        ));
    }
    for path in entries {
        if let Some(pattern) = paths.get(path) {
            if let Some(finding) = key(with.get("key"), path, pattern, location) {
                findings.push(finding);
            }
        }
    }
}

fn add_paths<'a>(value: &'a Value, entries: &mut BTreeSet<&'a str>) -> bool {
    let Some(path) = value.as_str() else {
        return false;
    };
    let mut known = true;
    for entry in path
        .lines()
        .map(normalized_path)
        .filter(|line| !line.is_empty())
    {
        if dynamic(entry) {
            known = false;
        } else {
            entries.insert(entry);
        }
    }
    known
}

fn key(
    value: Option<&Value>,
    path: &str,
    pattern: &Regex,
    location: &StepLocation<'_>,
) -> Option<RuleFinding> {
    let field = &["with", "key"];
    if let Some(text) = value.and_then(Value::as_str).map(str::trim) {
        if pattern.is_match(text) {
            return None;
        }
        if opaque_expression(text) {
            return Some(location.finding(
                field,
                "unanalyzable",
                &format!(
                    "cannot verify the cache key for `{path}`; write an explicit key template"
                ),
            ));
        }
    }
    Some(location.finding(
        field,
        "cache-key-shape",
        &format!("cache path `{path}` needs a key matching its configured keyPattern"),
    ))
}

fn dynamic(text: &str) -> bool {
    text.contains("${{")
}

fn opaque_expression(mut text: &str) -> bool {
    use crate::codebase::rules::github_actions_workflow_files::interpolated_expression_end;

    let mut interpolations = 0;
    let mut has_literal = false;
    while let Some(start) = text.find("${{") {
        has_literal |= start > 0;
        let body = &text[start + 3..];
        let Some(end) = interpolated_expression_end(body) else {
            return true;
        };
        interpolations += 1;
        text = &body[end + 2..];
    }
    interpolations == 1 && !has_literal && text.is_empty()
}
