//! Dependency names that count as a nested `optimizeDeps` declaration.

use super::specifier::package_name;
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn workspace_dependency_names(value: &Value) -> BTreeSet<String> {
    specifier_names(value, |specifier| specifier.starts_with("workspace:"))
}

pub(super) fn versioned_dependency_names(value: &Value) -> BTreeSet<String> {
    specifier_names(value, |specifier| !specifier.starts_with("workspace:"))
}

fn specifier_names(value: &Value, keep: impl Fn(&str) -> bool) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for field in ["dependencies", "peerDependencies", "optionalDependencies"] {
        let Some(Value::Object(map)) = value.get(field) else {
            continue;
        };
        for (key, specifier) in map {
            let Some(text) = specifier.as_str() else {
                continue;
            };
            let text = text.trim();
            if keep(text) {
                if let Some(package) = package_name(key) {
                    names.insert(package);
                }
            }
        }
    }
    names
}

pub(super) fn dependency_names(value: &Value) -> Result<BTreeSet<String>, String> {
    let mut names = BTreeSet::new();
    extend_objects(
        &mut names,
        value,
        &["dependencies", "peerDependencies", "optionalDependencies"],
    )?;
    extend_arrays(
        &mut names,
        value,
        &["bundledDependencies", "bundleDependencies"],
    )?;
    Ok(names)
}

fn extend_objects(
    names: &mut BTreeSet<String>,
    value: &Value,
    fields: &[&str],
) -> Result<(), String> {
    for field in fields {
        match value.get(*field) {
            None | Some(Value::Null) => {}
            Some(Value::Object(map)) => {
                names.extend(map.keys().filter_map(|key| package_name(key)));
            }
            Some(_) => return Err(format!("{field} must be an object")),
        }
    }
    Ok(())
}

fn extend_arrays(
    names: &mut BTreeSet<String>,
    value: &Value,
    fields: &[&str],
) -> Result<(), String> {
    for field in fields {
        match value.get(*field) {
            None | Some(Value::Null) => {}
            Some(Value::Bool(true)) => insert_object_names(names, value),
            Some(Value::Bool(false)) => {}
            Some(Value::Array(items)) => {
                for item in items {
                    let Some(name) = item.as_str() else {
                        continue;
                    };
                    if let Some(package) = package_name(name) {
                        names.insert(package);
                    }
                }
            }
            Some(_) => return Err(format!("{field} must be an array or a boolean")),
        }
    }
    Ok(())
}

fn insert_object_names(names: &mut BTreeSet<String>, value: &Value) {
    for field in ["dependencies", "peerDependencies", "optionalDependencies"] {
        if let Some(Value::Object(map)) = value.get(field) {
            names.extend(map.keys().filter_map(|key| package_name(key)));
        }
    }
}
