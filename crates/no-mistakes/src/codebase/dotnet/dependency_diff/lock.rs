use super::DotnetDependencyDiagnostic;
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) fn lock_projection(
    source: &str,
) -> Result<BTreeMap<String, String>, DotnetDependencyDiagnostic> {
    let value: Value =
        serde_json::from_str(source).map_err(|_| DotnetDependencyDiagnostic::MalformedLockfile)?;
    let version = value
        .get("version")
        .and_then(Value::as_u64)
        .ok_or(DotnetDependencyDiagnostic::UnsupportedLockSchema)?;
    if !(1..=2).contains(&version) {
        return Err(DotnetDependencyDiagnostic::UnsupportedLockSchema);
    }
    let dependencies = value
        .get("dependencies")
        .and_then(Value::as_object)
        .ok_or(DotnetDependencyDiagnostic::UnsupportedLockSchema)?;
    let mut records = BTreeMap::new();
    for (framework, packages) in dependencies {
        for (name, record) in packages
            .as_object()
            .ok_or(DotnetDependencyDiagnostic::UnsupportedLockSchema)?
        {
            let record = record
                .as_object()
                .ok_or(DotnetDependencyDiagnostic::UnsupportedLockSchema)?;
            record
                .get("resolved")
                .and_then(Value::as_str)
                .ok_or(DotnetDependencyDiagnostic::UnsupportedLockSchema)?;
            records.insert(
                format!("{framework}:{name}"),
                serde_json::to_string(record)
                    .expect("a parsed JSON object always serializes successfully"),
            );
        }
    }
    Ok(records)
}
