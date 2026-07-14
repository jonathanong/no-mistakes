use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackageDependency {
    pub field: String,
    pub name: String,
    pub specifier: String,
}

pub const ALL_DEPENDENCY_FIELDS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
];

pub const PRODUCTION_DEPENDENCY_FIELDS: &[&str] = &["dependencies", "optionalDependencies"];

pub fn dependency_entries(path: &Path, fields: &[&str]) -> Vec<PackageDependency> {
    let Ok(source) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&source) else {
        return Vec::new();
    };
    dependency_entries_from_value(&package_json, fields)
}

#[doc(hidden)]
pub fn dependency_entries_from_source_store(
    path: &Path,
    fields: &[&str],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<PackageDependency> {
    let Some(Ok(package_json)) = sources.parse_json_path(path) else {
        return Vec::new();
    };
    dependency_entries_from_value(&package_json, fields)
}

pub fn dependency_entries_from_value(
    package_json: &serde_json::Value,
    fields: &[&str],
) -> Vec<PackageDependency> {
    let mut entries = Vec::new();
    for field in fields {
        let Some(deps) = package_json.get(*field).and_then(|value| value.as_object()) else {
            continue;
        };
        for (name, version) in deps {
            let Some(specifier) = version.as_str() else {
                continue;
            };
            entries.push(PackageDependency {
                field: (*field).to_string(),
                name: name.clone(),
                specifier: specifier.to_string(),
            });
        }
    }
    entries.sort();
    entries.dedup();
    entries
}

pub fn dependency_names(path: &Path, fields: &[&str]) -> BTreeSet<String> {
    dependency_entries(path, fields)
        .into_iter()
        .map(|entry| entry.name)
        .collect()
}

pub fn dependency_names_from_value(
    package_json: &serde_json::Value,
    fields: &[&str],
) -> Vec<String> {
    dependency_entries_from_value(package_json, fields)
        .into_iter()
        .map(|entry| entry.name)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
