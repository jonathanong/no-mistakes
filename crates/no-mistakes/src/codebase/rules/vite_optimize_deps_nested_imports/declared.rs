//! Dependency names that count as a nested `optimizeDeps` declaration.

use super::specifier::package_name;
use serde_json::Value;
use std::collections::BTreeSet;

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
            Some(_) => return Err(format!("{field} must be an array")),
        }
    }
    Ok(())
}
