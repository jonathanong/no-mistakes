use super::value_at_key;
use super::ValueAssertion;
use crate::codebase::rules::structured_config_policy::paths::CanonicalInventory;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use serde_yaml::Value;
use std::path::Path;

mod extends;
mod keys;
mod lost;
mod matching;
mod spec;
pub(super) use extends::ParsedAncestorCache;
use extends::{collect_ancestors, Nested};
use keys::Keys;
use lost::lost_override_findings;

pub(super) fn check_ancestor_override_subset(
    nested_path: &Path,
    nested_rel: &str,
    sources: &SourceStore,
    value: &Value,
    assertion: &ValueAssertion,
    canonical_paths: &CanonicalInventory,
    parsed_ancestors: &mut ParsedAncestorCache,
) -> Vec<RuleFinding> {
    let keys = Keys::from_assertion(assertion);
    let mut findings = Vec::new();
    let Some(root) = canonical_paths.root() else {
        findings.push(finding(
            nested_rel,
            assertion,
            format!(
                "{nested_rel}: ancestor-override-subset cannot resolve the repository root safely"
            ),
        ));
        return findings;
    };
    let Some(nested_path) = canonical_paths.path(nested_path) else {
        findings.push(finding(
            nested_rel,
            assertion,
            format!(
                "{nested_rel}: ancestor-override-subset nested config is outside the repository root"
            ),
        ));
        return findings;
    };
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
        parsed_ancestors,
    );
    let nested_dir = nested_path.parent().unwrap_or(nested_path);
    let canonical_children: Vec<&Path> = canonical_paths
        .paths()
        .filter(|path| path != &nested_path && path.starts_with(nested_dir))
        .collect();
    let nested_rules = mapping_at(value, keys.rules);
    findings.extend(lost_override_findings(
        nested_rel,
        nested_dir,
        nested_rules,
        &canonical_children,
        &ancestors,
        assertion,
        &keys,
    ));
    findings
}

#[cfg(test)]
pub(super) fn parsed_ancestor_parse_count(cache: &ParsedAncestorCache, path: &Path) -> usize {
    cache.parse_count(path)
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
