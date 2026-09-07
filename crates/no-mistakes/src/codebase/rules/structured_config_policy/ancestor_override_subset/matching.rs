use super::Ancestor;
use crate::codebase::rules::structured_config_policy::ValueAssertion;
use globset::{Glob, GlobMatcher};
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) fn required_rules(
    ancestors: &[Ancestor],
    nested_path: &Path,
    candidates: &[PathBuf],
    assertion: &ValueAssertion,
) -> Result<BTreeMap<String, Value>, String> {
    let mut records = Vec::new();
    for ancestor in ancestors {
        records.extend(overrides(&ancestor.path, &ancestor.value, assertion)?);
    }
    let mut required = BTreeMap::new();
    let nested_dir = nested_path.parent().unwrap_or(nested_path);
    for candidate in candidates
        .iter()
        .filter(|candidate| candidate.strip_prefix(nested_dir).is_ok())
    {
        let mut before = BTreeMap::new();
        let mut after = BTreeMap::new();
        for record in &records {
            if record.matches(candidate, record.path.parent().unwrap_or(&record.path)) {
                before.extend(&record.rules);
            }
            if record.matches(candidate, nested_path.parent().unwrap_or(nested_path)) {
                after.extend(&record.rules);
            }
        }
        for (key, value) in before {
            if after.get(&key) != Some(&value) {
                required.insert(key.clone(), value.clone());
            }
        }
    }
    Ok(required)
}

pub(super) fn rules(value: &Value, key: &str) -> Result<BTreeMap<String, Value>, String> {
    let Some(value) = value.get(key) else {
        return Ok(BTreeMap::new());
    };
    let mapping = value
        .as_mapping()
        .ok_or_else(|| format!("`{key}` must be an object"))?;
    mapping
        .iter()
        .map(|(entry_key, value)| {
            entry_key
                .as_str()
                .map(|key| (key.to_string(), value.clone()))
                .ok_or_else(|| format!("`{key}` must have string keys"))
        })
        .collect()
}

struct OverrideRecord {
    path: PathBuf,
    includes: Vec<GlobMatcher>,
    excludes: Vec<GlobMatcher>,
    rules: BTreeMap<String, Value>,
}

impl OverrideRecord {
    fn matches(&self, candidate: &Path, base: &Path) -> bool {
        let Ok(relative) = candidate.strip_prefix(base) else {
            return false;
        };
        let relative = relative.to_string_lossy().replace('\\', "/");
        self.includes.iter().any(|glob| glob.is_match(&relative))
            && !self.excludes.iter().any(|glob| glob.is_match(&relative))
    }
}

fn overrides(
    path: &Path,
    value: &Value,
    assertion: &ValueAssertion,
) -> Result<Vec<OverrideRecord>, String> {
    let Some(value) = value.get(&assertion.overrides_key) else {
        return Ok(Vec::new());
    };
    let records = value
        .as_sequence()
        .ok_or_else(|| format!("`{}` must be an array", assertion.overrides_key))?;
    records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            let record = record.as_mapping().ok_or_else(|| {
                format!(
                    "`{}` entry {index} must be an object",
                    assertion.overrides_key
                )
            })?;
            Ok(OverrideRecord {
                path: path.to_path_buf(),
                includes: compile_patterns(record, &assertion.override_files_key, true)?,
                excludes: compile_patterns(record, &assertion.override_exclude_files_key, false)?,
                rules: rules(
                    &Value::Mapping(record.clone()),
                    &assertion.override_rules_key,
                )?,
            })
        })
        .collect()
}

pub(super) fn compile_patterns(
    record: &Mapping,
    key: &str,
    required: bool,
) -> Result<Vec<GlobMatcher>, String> {
    let Some(value) = record.get(Value::String(key.to_string())) else {
        return if required {
            Err(format!("override is missing required `{key}` patterns"))
        } else {
            Ok(Vec::new())
        };
    };
    let patterns = match value {
        Value::String(pattern) => vec![pattern.clone()],
        Value::Sequence(patterns) => patterns
            .iter()
            .map(|pattern| {
                pattern
                    .as_str()
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| format!("`{key}` must be a string or an array of strings"))
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err(format!("`{key}` must be a string or an array of strings")),
    };
    if required && patterns.is_empty() {
        return Err(format!("`{key}` must not be empty"));
    }
    patterns
        .into_iter()
        .map(|pattern| {
            Glob::new(&pattern)
                .map(|glob| glob.compile_matcher())
                .map_err(|error| format!("`{key}` has invalid glob `{pattern}`: {error}"))
        })
        .collect()
}
