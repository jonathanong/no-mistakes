use super::{ResolutionKind, ResolvedPackage};
use serde_json::{Map, Value};
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum NpmValidationError {
    UnsupportedVersion,
    Malformed(String),
}

impl fmt::Display for NpmValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion => f.write_str("unsupported npm lockfile version"),
            Self::Malformed(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for NpmValidationError {}

pub fn parse(content: &str) -> Vec<ResolvedPackage> {
    let Ok(root) = serde_json::from_str::<serde_json::Value>(content) else {
        return Vec::new();
    };
    parse_root(&root)
}

fn parse_root(root: &Value) -> Vec<ResolvedPackage> {
    let version = root
        .get("lockfileVersion")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    if version >= 2 {
        parse_v2(root)
    } else {
        parse_v1(root)
    }
}

pub(crate) fn parse_checked(
    content: &str,
) -> std::result::Result<Vec<ResolvedPackage>, NpmValidationError> {
    let root: Value = serde_json::from_str(content)
        .map_err(|_| NpmValidationError::Malformed("invalid npm lockfile JSON".into()))?;
    match root.get("lockfileVersion").and_then(Value::as_u64) {
        Some(1) if root.get("dependencies").is_none() => {}
        Some(1) => {
            let dependencies = root["dependencies"]
                .as_object()
                .ok_or_else(|| NpmValidationError::Malformed("invalid npm dependencies".into()))?;
            validate_records(dependencies, true)?;
        }
        Some(2 | 3) => {
            let packages = root
                .get("packages")
                .and_then(Value::as_object)
                .ok_or_else(|| NpmValidationError::Malformed("invalid npm packages".into()))?;
            validate_records(packages, false)?;
        }
        _ => return Err(NpmValidationError::UnsupportedVersion),
    }
    Ok(parse_root(&root))
}

fn validate_records(
    records: &Map<String, Value>,
    nested: bool,
) -> std::result::Result<(), NpmValidationError> {
    for (name, record) in records {
        if !record.is_object() {
            return Err(NpmValidationError::Malformed(format!(
                "invalid npm package record {name:?}"
            )));
        }
        if let Some(dependencies) = record.get("dependencies").filter(|_| nested) {
            let dependencies = dependencies.as_object().ok_or_else(|| {
                NpmValidationError::Malformed("invalid nested npm dependencies".into())
            })?;
            validate_records(dependencies, true)?;
        }
    }
    Ok(())
}

fn parse_v2(root: &serde_json::Value) -> Vec<ResolvedPackage> {
    let Some(packages) = root.get("packages").and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    packages
        .iter()
        .filter(|(k, _)| !k.is_empty())
        .map(|(key, value)| {
            let name = if key.starts_with("node_modules/") {
                // Use the import name from the key, not the name field.
                // The name field may differ for aliases (e.g., key "node_modules/foo",
                // name "lodash" for "foo": "npm:lodash@4") — imports use the alias.
                key.trim_start_matches("node_modules/")
                    .split("/node_modules/")
                    .last()
                    .unwrap_or(key.as_str())
                    .to_string()
            } else if let Some((_, after)) = key.split_once("/node_modules/") {
                // Nested workspace install (e.g., "packages/app/node_modules/foo").
                // The import specifier is the part after the last /node_modules/ segment.
                after
                    .split("/node_modules/")
                    .last()
                    .unwrap_or(after)
                    .to_string()
            } else {
                // Workspace package root (e.g., "packages/lib") — use the declared package name
                value
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(key.as_str())
                    .to_string()
            };
            let version = value
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let integrity = value
                .get("integrity")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let resolved = value.get("resolved").and_then(|v| v.as_str()).unwrap_or("");
            let kind = if value.get("link").and_then(|v| v.as_bool()).unwrap_or(false) {
                ResolutionKind::Workspace
            } else if resolved.is_empty() {
                ResolutionKind::Directory
            } else {
                ResolutionKind::Registry
            };
            let fingerprint = if integrity.is_empty() {
                resolved.to_string()
            } else {
                integrity.to_string()
            };
            ResolvedPackage {
                name,
                version,
                fingerprint,
                kind,
            }
        })
        .collect()
}

fn parse_v1(root: &serde_json::Value) -> Vec<ResolvedPackage> {
    let Some(deps) = root.get("dependencies").and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    collect_v1_deps(deps)
}

fn collect_v1_deps(deps: &serde_json::Map<String, serde_json::Value>) -> Vec<ResolvedPackage> {
    let mut result = Vec::new();
    for (name, value) in deps {
        let version = value
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let integrity = value
            .get("integrity")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let resolved = value.get("resolved").and_then(|v| v.as_str()).unwrap_or("");
        let fingerprint = if integrity.is_empty() {
            resolved.to_string()
        } else {
            integrity.to_string()
        };
        result.push(ResolvedPackage {
            name: name.clone(),
            version,
            fingerprint,
            kind: ResolutionKind::Registry,
        });
        if let Some(nested) = value.get("dependencies").and_then(|v| v.as_object()) {
            result.extend(collect_v1_deps(nested));
        }
    }
    result
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
