use super::value_assertions::selector::values_at_selector;
use super::{ValueAssertion, RULE_ID};
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
    if other_path.strip_prefix(root).is_err() {
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
    let left = values_at_selector(value, &assertion.key);
    let right = values_at_selector(&other, from_key);
    if !left.has_missing && !right.has_missing && left.values == right.values {
        return Vec::new();
    }
    vec![finding(
        rel,
        assertion,
        format!(
            "{rel}: config value `{}` must equal `{}` from `{}`",
            assertion.key, from_key, assertion.file
        ),
    )]
}

fn finding(file: &str, assertion: &ValueAssertion, fallback: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message: assertion.message.clone().unwrap_or(fallback),
        import: None,
        target: Some(assertion.key.clone()),
    }
}
