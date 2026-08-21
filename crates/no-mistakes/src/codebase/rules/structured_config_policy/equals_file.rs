use super::value_assertions::selector::{any_groups, values_at_selector};
use super::{MatchMode, ValueAssertion, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::codebase::structured_value::parse_structured_value;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use serde_yaml::Value;
use std::path::Path;

pub(super) fn check_equals_file(
    root: &Path,
    rel: &str,
    sources: &SourceStore,
    value: &Value,
    assertion: &ValueAssertion,
) -> Vec<RuleFinding> {
    if assertion.key.is_empty() {
        return vec![finding(
            rel,
            assertion,
            format!("{rel}: equals-file assertion is missing `key`"),
        )];
    }
    if assertion.file.is_empty() {
        return vec![finding(
            rel,
            assertion,
            format!("{rel}: equals-file assertion is missing `file`"),
        )];
    }
    let from_key = if assertion.from_key.is_empty() {
        assertion.key.as_str()
    } else {
        assertion.from_key.as_str()
    };
    let other_path = normalize_path(&root.join(&assertion.file));
    if !contained_in_root(root, &other_path) {
        return vec![finding(
            rel,
            assertion,
            format!(
                "{rel}: equals-file `{}` is outside the repository root",
                assertion.file
            ),
        )];
    }
    let other_rel = relative_slash_path(root, &other_path);
    let Some(source) = super::super::read_source(sources, &other_path) else {
        return vec![finding(
            rel,
            assertion,
            format!("{rel}: equals-file `{}` is missing", assertion.file),
        )];
    };
    let other = match parse_structured_value(&other_path, &source) {
        Ok(other) => other,
        Err(error) => {
            return vec![finding(
                &other_rel,
                assertion,
                format!("{other_rel}: {error}"),
            )];
        }
    };
    if values_match(value, &other, assertion, from_key) {
        return Vec::new();
    }
    vec![finding(
        rel,
        assertion,
        assertion.message.clone().unwrap_or_else(|| {
            format!(
                "{rel}: config value `{}` must equal `{}` from `{}`",
                assertion.key, from_key, assertion.file
            )
        }),
    )]
}

fn values_match(left: &Value, right: &Value, assertion: &ValueAssertion, from_key: &str) -> bool {
    let expected = values_at_selector(right, from_key);
    if expected.has_missing {
        return false;
    }
    if assertion.match_mode == MatchMode::Any {
        return !any_groups(left, &assertion.key)
            .into_iter()
            .any(|group| !group.iter().any(|value| expected.values.contains(value)));
    }
    let actual = values_at_selector(left, &assertion.key);
    !actual.has_missing && actual.values == expected.values
}

fn contained_in_root(root: &Path, path: &Path) -> bool {
    if path.strip_prefix(root).is_err() {
        return false;
    }
    match (path.canonicalize(), root.canonicalize()) {
        (Ok(resolved), Ok(resolved_root)) => resolved.strip_prefix(resolved_root).is_ok(),
        (Err(_), _) => true,
        (Ok(resolved), Err(_)) => resolved.strip_prefix(root).is_ok(),
    }
}

fn finding(file: &str, assertion: &ValueAssertion, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message,
        import: None,
        target: Some(assertion.key.clone()),
    }
}
