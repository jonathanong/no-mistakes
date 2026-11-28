use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc(hidden)]
pub struct NugetLockError;

#[doc(hidden)]
pub fn nuget_entries(root: &Value) -> Result<BTreeMap<String, Value>, NugetLockError> {
    if !matches!(root.get("version").and_then(Value::as_u64), Some(1 | 2)) {
        return Err(NugetLockError);
    }
    let dependencies = root
        .get("dependencies")
        .and_then(Value::as_object)
        .ok_or(NugetLockError)?;
    let mut entries = BTreeMap::new();
    for (framework, packages) in dependencies {
        for (name, record) in packages.as_object().ok_or(NugetLockError)? {
            let record = record.as_object().ok_or(NugetLockError)?;
            if record.get("type").and_then(Value::as_str) != Some("Project") {
                record
                    .get("resolved")
                    .and_then(Value::as_str)
                    .ok_or(NugetLockError)?;
            }
            entries.insert(format!("{framework}:{name}"), Value::Object(record.clone()));
        }
    }
    Ok(entries)
}
