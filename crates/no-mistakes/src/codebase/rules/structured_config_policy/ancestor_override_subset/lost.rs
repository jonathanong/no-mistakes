use super::extends::{Ancestor, Keys};
use super::{finding, mapping_at, string_entries};
use crate::codebase::rules::structured_config_policy::value_at_key;
use crate::codebase::rules::structured_config_policy::ValueAssertion;
use crate::codebase::rules::RuleFinding;
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde_yaml::Mapping;
use std::path::Path;

pub(super) fn lost_override_findings(
    nested_rel: &str,
    nested_dir: &Path,
    nested_rules: Option<&Mapping>,
    children: &[&Path],
    ancestor: &Ancestor,
    assertion: &ValueAssertion,
    keys: &Keys<'_>,
) -> Vec<RuleFinding> {
    let Some(overrides) = value_at_key(&ancestor.value, keys.overrides) else {
        return Vec::new();
    };
    let Some(overrides) = overrides.as_sequence() else {
        return Vec::new();
    };
    let ancestor_dir = ancestor.path.parent().unwrap_or(&ancestor.path);
    let mut findings = Vec::new();
    for override_value in overrides {
        let Some(override_rules) = mapping_at(override_value, keys.rules) else {
            continue;
        };
        if override_rules.is_empty() {
            continue;
        }
        let Some(globs) = compile_globs(&string_entries(value_at_key(override_value, keys.files)))
        else {
            continue;
        };
        if !any_child_matches(&globs, children, ancestor_dir) {
            continue;
        }
        if any_child_matches(&globs, children, nested_dir) {
            continue;
        }
        if rules_are_subset(override_rules, nested_rules) {
            continue;
        }
        findings.push(finding(
            nested_rel,
            assertion,
            assertion.message.clone().unwrap_or_else(|| {
                format!(
                    "{nested_rel}: lost ancestor override from `{}` must be a value-equal subset of `{}`",
                    ancestor.rel, keys.rules
                )
            }),
        ));
    }
    findings
}

fn compile_globs(patterns: &[&str]) -> Option<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    let mut any = false;
    for pattern in patterns {
        let trimmed = pattern.trim_start_matches("./");
        if let Ok(glob) = Glob::new(trimmed) {
            builder.add(glob);
            any = true;
        }
    }
    if !any {
        return None;
    }
    builder.build().ok()
}

fn any_child_matches(globs: &GlobSet, children: &[&Path], base_dir: &Path) -> bool {
    children
        .iter()
        .copied()
        .any(|child| relative_under(base_dir, child).is_some_and(|rel| globs.is_match(&rel)))
}

fn relative_under(base_dir: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(base_dir)
        .ok()
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
}

fn rules_are_subset(override_rules: &Mapping, nested_rules: Option<&Mapping>) -> bool {
    let Some(nested_rules) = nested_rules else {
        return false;
    };
    override_rules.iter().all(|(key, expected)| {
        nested_rules
            .get(key)
            .is_some_and(|actual| actual == expected)
    })
}
