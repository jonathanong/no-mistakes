use super::value_at_key;
use super::ValueAssertion;
use crate::codebase::rules::structured_config_policy::paths::canonical_path_in_root;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use serde_yaml::Value;
use std::path::{Path, PathBuf};

mod extends;
mod keys;
mod lost;
mod matching;
mod spec;
use extends::{collect_ancestors, Nested};
use keys::Keys;
use lost::lost_override_findings;

pub(super) fn check_ancestor_override_subset(
    root: &Path,
    nested_path: &Path,
    nested_rel: &str,
    sources: &SourceStore,
    files: &[PathBuf],
    value: &Value,
    assertion: &ValueAssertion,
) -> Vec<RuleFinding> {
    let keys = Keys::from_assertion(assertion);
    let mut findings = Vec::new();
    let ancestors = collect_ancestors(
        root,
        Nested {
            path: nested_path,
            rel: nested_rel,
            value,
        },
        sources,
        assertion,
        &keys,
        &mut findings,
    );
    let Some(nested_path) = canonical_path_in_root(root, nested_path) else {
        return findings;
    };
    let nested_dir = nested_path.parent().unwrap_or(&nested_path);
    let canonical_children: Vec<PathBuf> = files
        .iter()
        .filter_map(|path| canonical_path_in_root(root, path))
        .filter(|path| path != &nested_path && path.starts_with(nested_dir))
        .collect();
    let children = canonical_children
        .iter()
        .map(PathBuf::as_path)
        .collect::<Vec<_>>();
    let nested_rules = mapping_at(value, keys.rules);
    findings.extend(lost_override_findings(
        nested_rel,
        nested_dir,
        nested_rules,
        &children,
        &ancestors,
        assertion,
        &keys,
    ));
    findings
}

pub(super) fn mapping_at<'a>(value: &'a Value, key: &str) -> Option<&'a serde_yaml::Mapping> {
    value_at_key(value, key).and_then(Value::as_mapping)
}

pub(super) fn finding(file: &str, assertion: &ValueAssertion, message: String) -> RuleFinding {
    let target = if assertion.key.is_empty() {
        if assertion.rules_key.is_empty() {
            "rules".to_string()
        } else {
            assertion.rules_key.clone()
        }
    } else {
        assertion.key.clone()
    };
    crate::codebase::rules::RuleFinding {
        rule: super::RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message,
        import: None,
        target: Some(target),
    }
}
