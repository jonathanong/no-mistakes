use super::{value_at_key, ValueAssertion, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::codebase::structured_value::parse_structured_value;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::SourceStore;
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
            return vec![finding(rel, assertion, format!("{rel}: {error}"))];
        }
    };
    if value_at_key(value, &assertion.key) == value_at_key(&other, from_key) {
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

fn finding(rel: &str, assertion: &ValueAssertion, fallback: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line: 1,
        message: assertion.message.clone().unwrap_or(fallback),
        import: None,
        target: Some(assertion.key.clone()),
    }
}
