use super::extends::Ancestor;
use super::finding;
use super::keys::Keys;
use super::matching::{compile_value_globs, mapping_value, optional_value_globs, relative_path};
use crate::codebase::rules::structured_config_policy::value_at_key;
use crate::codebase::rules::structured_config_policy::ValueAssertion;
use crate::codebase::rules::RuleFinding;
use globset::GlobSet;
use serde_yaml::{Mapping, Value};
use std::path::Path;

struct Override<'a> {
    rules: &'a Mapping,
    files: GlobSet,
    exclude_files: Option<GlobSet>,
    dir: &'a Path,
}

pub(super) fn lost_override_findings(
    nested_rel: &str,
    nested_dir: &Path,
    nested_rules: Option<&Mapping>,
    children: &[&Path],
    ancestors: &[Ancestor],
    assertion: &ValueAssertion,
    keys: &Keys<'_>,
) -> Vec<RuleFinding> {
    let (overrides, mut findings) = collect_overrides(ancestors, nested_rel, assertion, keys);
    let has_lost_rules = children.iter().copied().any(|child| {
        let before = effective_rules(&overrides, child, |override_| override_.dir);
        let after = effective_rules(&overrides, child, |_| nested_dir);
        let lost = before
            .into_iter()
            .filter(|(key, value)| lookup(&after, key) != Some(value))
            .collect::<Vec<_>>();
        !lost.is_empty() && !rules_are_subset(&lost, nested_rules)
    });
    if has_lost_rules {
        findings.push(finding(
            nested_rel,
            assertion,
            assertion.message.clone().unwrap_or_else(|| {
                format!(
                    "{nested_rel}: lost ancestor override must be a value-equal subset of `{}`",
                    keys.rules
                )
            }),
        ));
    }
    findings
}

fn collect_overrides<'a>(
    ancestors: &'a [Ancestor],
    nested_rel: &str,
    assertion: &ValueAssertion,
    keys: &Keys<'_>,
) -> (Vec<Override<'a>>, Vec<RuleFinding>) {
    let mut collected = Vec::new();
    let mut findings = Vec::new();
    for ancestor in ancestors {
        let Some(overrides) = value_at_key(&ancestor.value, keys.overrides) else {
            continue;
        };
        let Some(overrides) = overrides.as_sequence() else {
            findings.push(invalid_override(
                nested_rel,
                assertion,
                &ancestor.rel,
                "must be an array",
            ));
            continue;
        };
        for override_value in overrides {
            let Some(override_value) = override_value.as_mapping() else {
                findings.push(invalid_override(
                    nested_rel,
                    assertion,
                    &ancestor.rel,
                    "must be an object",
                ));
                continue;
            };
            let Some(rules) = mapping_value(override_value, keys.rules).and_then(Value::as_mapping)
            else {
                if mapping_value(override_value, keys.rules).is_some() {
                    findings.push(invalid_override(
                        nested_rel,
                        assertion,
                        &ancestor.rel,
                        "rules must be an object",
                    ));
                }
                continue;
            };
            if rules.is_empty() {
                continue;
            }
            let Some(files) = compile_value_globs(override_value, keys.files) else {
                findings.push(invalid_override(
                    nested_rel,
                    assertion,
                    &ancestor.rel,
                    "files must contain valid string globs",
                ));
                continue;
            };
            let exclude_files = match optional_value_globs(override_value, keys.exclude_files) {
                Ok(exclude_files) => exclude_files,
                Err(()) => {
                    findings.push(invalid_override(
                        nested_rel,
                        assertion,
                        &ancestor.rel,
                        "excludeFiles must contain valid string globs",
                    ));
                    continue;
                }
            };
            collected.push(Override {
                rules,
                files,
                exclude_files,
                dir: ancestor.path.parent().unwrap_or(&ancestor.path),
            });
        }
    }
    (collected, findings)
}

fn invalid_override(
    nested_rel: &str,
    assertion: &ValueAssertion,
    ancestor_rel: &str,
    detail: &str,
) -> RuleFinding {
    finding(
        nested_rel,
        assertion,
        format!("{nested_rel}: ancestor override in `{ancestor_rel}` {detail}"),
    )
}

fn effective_rules<'a>(
    overrides: &'a [Override<'a>],
    child: &Path,
    base_dir: impl Fn(&Override<'a>) -> &'a Path,
) -> Vec<(&'a Value, &'a Value)> {
    let mut rules = Vec::new();
    for override_ in overrides {
        let Some(rel) = relative_path(base_dir(override_), child) else {
            continue;
        };
        if !override_.files.is_match(&rel)
            || override_
                .exclude_files
                .as_ref()
                .is_some_and(|exclude| exclude.is_match(&rel))
        {
            continue;
        }
        for (key, value) in override_.rules {
            set_rule(&mut rules, key, value);
        }
    }
    rules
}

fn set_rule<'a>(rules: &mut Vec<(&'a Value, &'a Value)>, key: &'a Value, value: &'a Value) {
    if let Some((_, existing)) = rules.iter_mut().find(|(current, _)| *current == key) {
        *existing = value;
    } else {
        rules.push((key, value));
    }
}

fn lookup<'a>(rules: &'a [(&Value, &'a Value)], key: &Value) -> Option<&'a Value> {
    rules
        .iter()
        .find_map(|(current, value)| (*current == key).then_some(*value))
}

fn rules_are_subset(expected: &[(&Value, &Value)], nested_rules: Option<&Mapping>) -> bool {
    let Some(nested_rules) = nested_rules else {
        return false;
    };
    expected
        .iter()
        .all(|(key, value)| nested_rules.get(*key) == Some(*value))
}
