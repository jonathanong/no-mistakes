use super::paths::contained_in_root;
use super::value_assertions::selector::{any_groups, values_at_selector};
use super::{MatchMode, RULE_ID, ValueAssertion};
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::structured_value::parse_structured_value_located;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{SourceStore, relative_slash_path};
use serde_yaml::Value;
use std::path::Path;

pub(super) fn check_equals_file(
    root: &Path,
    rel: &str,
    sources: &SourceStore,
    value: &Value,
    assertion: &ValueAssertion,
    uncertainty: &mut Vec<RuleFinding>,
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
    let Some(source) = super::super::read_language_source(sources, &other_path) else {
        return vec![finding(
            rel,
            assertion,
            format!("{rel}: equals-file `{}` is missing", assertion.file),
        )];
    };
    let parsed = parse_structured_value_located(&other_path, &source);
    let other = match &parsed {
        Ok(other) => other,
        Err(error) if error.missing_merge_location => {
            uncertainty.push(super::merge_location_finding(root, &other_path, error));
            let Some(partial) = error.partial.as_deref() else {
                return Vec::new();
            };
            if !super::partial::selector_known(partial, from_key) {
                return Vec::new();
            }
            partial.value()
        }
        Err(error) => {
            return vec![finding(
                &other_rel,
                assertion,
                format!("{other_rel}: {error}"),
            )];
        }
    };
    if values_match(value, other, assertion, from_key) {
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

fn finding(file: &str, assertion: &ValueAssertion, message: String) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message,
        import: None,
        target: Some(assertion.key.clone()),
    }
}
