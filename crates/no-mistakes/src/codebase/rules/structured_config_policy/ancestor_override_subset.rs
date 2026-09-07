use super::{ValueAssertion, RULE_ID};
use crate::codebase::rules::RuleFinding;
use serde_yaml::Value;
use std::path::{Path, PathBuf};

mod matching;
mod resolution;

pub(crate) use resolution::AncestorResolver;

pub(super) fn check_ancestor_override_subset(
    rel: &str,
    path: &Path,
    candidates: &[PathBuf],
    value: &Value,
    assertion: &ValueAssertion,
    resolver: &mut AncestorResolver<'_>,
) -> Vec<RuleFinding> {
    let ancestors = match resolver.resolve(path, value, assertion) {
        Ok(ancestors) => ancestors,
        Err(message) => return vec![finding(rel, assertion, message)],
    };
    let required = match matching::required_rules(&ancestors, path, candidates, assertion) {
        Ok(required) => required,
        Err(message) => return vec![finding(rel, assertion, format!("{rel}: {message}"))],
    };
    let nested_rules = match matching::rules(value, &assertion.override_rules_key) {
        Ok(rules) => rules,
        Err(message) => return vec![finding(rel, assertion, format!("{rel}: {message}"))],
    };
    let missing = required
        .iter()
        .filter(|(key, expected)| nested_rules.get(*key) != Some(expected))
        .map(|(key, _)| key.as_str())
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return Vec::new();
    }
    vec![finding(
        rel,
        assertion,
        assertion.message.clone().unwrap_or_else(|| {
            format!(
                "{rel}: nested `{}` must restate lost ancestor override rules: {}",
                assertion.override_rules_key,
                missing.join(", ")
            )
        }),
    )]
}

pub(super) fn finding(file: &str, assertion: &ValueAssertion, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message,
        import: None,
        target: Some(assertion.key.clone()),
    }
}

#[derive(Clone)]
pub(super) struct Ancestor {
    pub(super) path: PathBuf,
    pub(super) value: Value,
}
