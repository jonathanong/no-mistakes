//! Static `secrets.*` use sites attributed to topology node scopes.
//!
//! This module extracts names from authored expressions only. It never
//! contacts GitHub, reads process environment values, or resolves secret
//! material.

use super::expression_references::static_context_references;
use serde_yaml::Value;
use std::collections::BTreeMap;

pub fn workflow(value: &Value) -> Option<Vec<String>> {
    let mapping = value.as_mapping()?;
    collect_mapping(mapping, &["jobs", "on"], None)
}

pub fn job(value: &Value) -> Option<Vec<String>> {
    let mapping = value.as_mapping()?;
    collect_mapping(mapping, &["steps"], value.get("if"))
}

pub fn step(value: &Value) -> Option<Vec<String>> {
    let mapping = value.as_mapping()?;
    collect_mapping(mapping, &[], value.get("if"))
}

fn collect_mapping(
    mapping: &serde_yaml::Mapping,
    excluded_keys: &[&str],
    bare_condition: Option<&Value>,
) -> Option<Vec<String>> {
    let mut references = BTreeMap::<String, String>::new();
    for (key, value) in mapping {
        if key.as_str().is_some_and(|key| excluded_keys.contains(&key)) {
            continue;
        }
        visit(value, &mut references);
    }
    if let Some(condition) = bare_condition.and_then(Value::as_str) {
        merge(
            &mut references,
            static_context_references(Some(condition), "secrets", true),
        );
    }
    (!references.is_empty()).then(|| references.into_values().collect())
}

fn visit(value: &Value, references: &mut BTreeMap<String, String>) {
    match value {
        Value::String(text) => merge(
            references,
            static_context_references(Some(text), "secrets", false),
        ),
        Value::Sequence(items) => {
            for item in items {
                visit(item, references);
            }
        }
        Value::Mapping(mapping) => {
            for value in mapping.values() {
                visit(value, references);
            }
        }
        _ => {}
    }
}

fn merge(references: &mut BTreeMap<String, String>, found: Vec<String>) {
    for name in found {
        references.entry(name.to_ascii_lowercase()).or_insert(name);
    }
}
