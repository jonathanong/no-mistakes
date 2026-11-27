use super::ValueAssertion;
use super::{partial, value_at_key};
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::rules::file_diagnostics::ClassifiedInputFindings;
use crate::codebase::rules::structured_config_policy::paths::CanonicalInventory;
use crate::codebase::structured_value::PartialYaml;
use crate::codebase::ts_source::SourceStore;
use serde_yaml::Value;
use std::path::Path;

mod extends;
mod keys;
mod lost;
mod matching;
mod spec;
pub(in crate::codebase::rules::structured_config_policy) use extends::ParsedAncestorCache;
use extends::{Nested, collect_ancestors};
use keys::Keys;
use lost::lost_override_findings;

pub(super) fn inputs_known(partial: &PartialYaml, assertion: &ValueAssertion) -> bool {
    let keys = Keys::from_assertion(assertion);
    [keys.extends, keys.rules].into_iter().all(|path| {
        partial::key(partial.value(), path, Some(partial))
            .is_some_and(|value| value.is_none_or(|value| !partial.contains_unknown(value)))
    })
}

pub(super) fn check_ancestor_override_subset(
    nested_path: &Path,
    nested_rel: &str,
    sources: &SourceStore,
    value: &Value,
    assertion: &ValueAssertion,
    canonical_paths: &CanonicalInventory,
    parsed_ancestors: &mut ParsedAncestorCache,
) -> ClassifiedInputFindings {
    let keys = Keys::from_assertion(assertion);
    let mut result = ClassifiedInputFindings::default();
    let Some(root) = canonical_paths.root() else {
        result.findings.push(finding(
            nested_rel,
            assertion,
            format!(
                "{nested_rel}: ancestor-override-subset cannot resolve the repository root safely"
            ),
        ));
        return result;
    };
    let Some(nested_path) = canonical_paths.path(nested_path) else {
        result.findings.push(finding(
            nested_rel,
            assertion,
            format!(
                "{nested_rel}: ancestor-override-subset nested config is outside the repository root"
            ),
        ));
        return result;
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
        &mut result,
        parsed_ancestors,
    );
    let nested_dir = nested_path.parent().unwrap_or(nested_path);
    let canonical_children: Vec<&Path> = canonical_paths
        .paths()
        .filter(|path| path != &nested_path && path.starts_with(nested_dir))
        .collect();
    let nested_rules = mapping_at(value, keys.rules);
    result.findings.extend(lost_override_findings(
        nested_rel,
        nested_dir,
        nested_rules,
        &canonical_children,
        &ancestors,
        assertion,
        &keys,
    ));
    result
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
    no_mistakes_rule_kernel::finding::RuleFinding {
        causes: Vec::new(),
        rule: super::RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message,
        import: None,
        target: Some(target),
    }
}

pub(super) fn validate_partial_extends(
    partial: &PartialYaml,
    rel: &str,
    assertion: &ValueAssertion,
) -> Option<RuleFinding> {
    let keys = Keys::from_assertion(assertion);
    partial::key(partial.value(), keys.extends, Some(partial))??;
    let message = spec::extends_specs(partial.value(), keys.extends).err()?;
    Some(finding(
        rel,
        assertion,
        format!("{rel}: ancestor-override-subset extends {message}"),
    ))
}
