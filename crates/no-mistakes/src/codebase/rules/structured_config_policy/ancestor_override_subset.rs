use super::{ValueAssertion, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::codebase::structured_value::parse_structured_value;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use globset::Glob;
use serde_yaml::{Mapping, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) fn check_ancestor_override_subset(
    root: &Path,
    rel: &str,
    path: &Path,
    sources: &SourceStore,
    value: &Value,
    assertion: &ValueAssertion,
) -> Vec<RuleFinding> {
    let ancestors = match ancestors(root, path, sources, value, assertion, &mut BTreeSet::new()) {
        Ok(ancestors) => ancestors,
        Err(message) => return vec![finding(rel, assertion, message)],
    };
    let nested_rules = rules(value, &assertion.override_rules_key);
    let mut required = BTreeMap::new();
    for ancestor in ancestors {
        for override_record in overrides(&ancestor.value, assertion) {
            if override_record_applies_before_but_not_after(
                &ancestor.path,
                path,
                &override_record.files,
            ) {
                required.extend(override_record.rules);
            }
        }
    }
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

struct Ancestor {
    path: PathBuf,
    value: Value,
}

struct OverrideRecord {
    files: Vec<String>,
    rules: BTreeMap<String, Value>,
}

fn ancestors(
    root: &Path,
    path: &Path,
    sources: &SourceStore,
    value: &Value,
    assertion: &ValueAssertion,
    visiting: &mut BTreeSet<PathBuf>,
) -> Result<Vec<Ancestor>, String> {
    let path = normalize_path(path);
    if !visiting.insert(path.clone()) {
        return Err(format!(
            "{}: `{}` contains an extends cycle",
            relative_slash_path(root, &path),
            assertion.extends_key
        ));
    }
    let result = (|| {
        let mut resolved = Vec::new();
        for specifier in extends(value, &assertion.extends_key)? {
            if !is_local_relative(&specifier) {
                continue;
            }
            let parent = normalize_path(
                &path
                    .parent()
                    .unwrap_or(root)
                    .join(specifier.replace('\\', "/")),
            );
            if !contained_in_root(root, &parent) {
                return Err(format!(
                    "{}: `{}` reference is outside the repository root: {specifier}",
                    relative_slash_path(root, &path),
                    assertion.extends_key
                ));
            }
            let parent_rel = relative_slash_path(root, &parent);
            let source = super::super::read_source(sources, &parent).ok_or_else(|| {
                format!(
                    "{}: `{}` reference is missing: {specifier}",
                    relative_slash_path(root, &path),
                    assertion.extends_key
                )
            })?;
            let parent_value = parse_structured_value(&parent, &source)
                .map_err(|error| format!("{parent_rel}: {error}"))?;
            resolved.extend(ancestors(
                root,
                &parent,
                sources,
                &parent_value,
                assertion,
                visiting,
            )?);
            resolved.push(Ancestor {
                path: parent,
                value: parent_value,
            });
        }
        Ok(resolved)
    })();
    visiting.remove(&path);
    result
}

fn extends(value: &Value, key: &str) -> Result<Vec<String>, String> {
    let Some(value) = value.get(key) else {
        return Ok(Vec::new());
    };
    match value {
        Value::String(path) => Ok(vec![path.clone()]),
        Value::Sequence(paths) => paths
            .iter()
            .map(|path| match path {
                Value::String(path) => Ok(path.clone()),
                _ => Err(format!("`{key}` must be a string or an array of strings")),
            })
            .collect(),
        _ => Err(format!("`{key}` must be a string or an array of strings")),
    }
}

fn is_local_relative(specifier: &str) -> bool {
    specifier.starts_with("./") || specifier.starts_with("../")
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

fn overrides(value: &Value, assertion: &ValueAssertion) -> Vec<OverrideRecord> {
    value
        .get(&assertion.overrides_key)
        .and_then(Value::as_sequence)
        .into_iter()
        .flatten()
        .filter_map(Value::as_mapping)
        .filter_map(|record| {
            let files = strings(record, &assertion.override_files_key);
            (!files.is_empty()).then(|| OverrideRecord {
                files,
                rules: rules(
                    &Value::Mapping(record.clone()),
                    &assertion.override_rules_key,
                ),
            })
        })
        .collect()
}

fn strings(record: &Mapping, key: &str) -> Vec<String> {
    match record.get(Value::String(key.to_string())) {
        Some(Value::String(value)) => vec![value.clone()],
        Some(Value::Sequence(values)) => values
            .iter()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect(),
        _ => Vec::new(),
    }
}

fn rules(value: &Value, key: &str) -> BTreeMap<String, Value> {
    value
        .get(key)
        .and_then(Value::as_mapping)
        .into_iter()
        .flatten()
        .filter_map(|(key, value)| key.as_str().map(|key| (key.to_string(), value.clone())))
        .collect()
}

fn override_record_applies_before_but_not_after(
    ancestor_path: &Path,
    nested_path: &Path,
    patterns: &[String],
) -> bool {
    let ancestor_dir = ancestor_path.parent().unwrap_or(ancestor_path);
    let nested_dir = nested_path.parent().unwrap_or(nested_path);
    let Ok(before) = nested_dir.strip_prefix(ancestor_dir) else {
        return false;
    };
    let before = before.join("__no_mistakes_override_probe__.ts");
    let after = Path::new("__no_mistakes_override_probe__.ts");
    patterns.iter().any(|pattern| {
        Glob::new(pattern).is_ok_and(|glob| {
            let matcher = glob.compile_matcher();
            matcher.is_match(&before) && !matcher.is_match(after)
        })
    })
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
