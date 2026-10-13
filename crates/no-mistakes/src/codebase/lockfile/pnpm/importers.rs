use super::{split_name_version, yaml_key_to_string};
use crate::codebase::lockfile::pnpm_alias::valid_package_name;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PnpmImporter {
    pub path: String,
    pub dependencies: Vec<PnpmImporterDependency>,
    pub dev_dependencies: Vec<PnpmImporterDependency>,
    pub optional_dependencies: Vec<PnpmImporterDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct PnpmImpactImporter {
    pub(super) path: String,
    pub(super) dependencies: Vec<PnpmImporterDependency>,
    pub(super) dev_dependencies: Vec<PnpmImporterDependency>,
    pub(super) optional_dependencies: Vec<PnpmImporterDependency>,
    pub(super) peer_dependencies: Vec<PnpmImporterDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PnpmImporterDependency {
    pub alias: String,
    pub resolution_name: Option<String>,
    pub specifier: String,
    pub version: String,
}

pub fn parse_importers(content: &str) -> Vec<PnpmImporter> {
    let Ok(docs) = super::documents::load_documents(content) else {
        return Vec::new();
    };
    parse_importers_from_documents(&docs)
}

/// Project workspace importers from documents a caller has already parsed.
/// This lets request-scoped consumers share one YAML parse for validation
/// and importer extraction.
pub(super) fn parse_importers_from_documents(
    docs: &super::documents::PnpmDocuments,
) -> Vec<PnpmImporter> {
    parse_importers_from_project(docs)
        .into_iter()
        .map(|importer| PnpmImporter {
            path: importer.path,
            dependencies: importer.dependencies,
            dev_dependencies: importer.dev_dependencies,
            optional_dependencies: importer.optional_dependencies,
        })
        .collect()
}

pub(crate) fn parse_importers_for_impact(content: &str) -> Vec<PnpmImpactImporter> {
    // Importers come from the project document. The env document reuses `.`
    // for config and package-manager dependencies and is not a workspace root.
    let Ok(docs) = super::documents::load_documents(content) else {
        return Vec::new();
    };
    parse_importers_from_project(&docs)
}

fn parse_importers_from_project(docs: &super::documents::PnpmDocuments) -> Vec<PnpmImpactImporter> {
    // Importers come from the project document. The env document reuses `.`
    // for config and package-manager dependencies, so it is not a workspace root.
    let Some(root) = docs.project() else {
        return Vec::new();
    };
    let Some(importers_map) = root.get("importers").and_then(|v| v.as_mapping()) else {
        return Vec::new();
    };

    let mut importers: Vec<PnpmImpactImporter> = importers_map
        .iter()
        .filter_map(|(key, value)| {
            let path = yaml_key_to_string(key);
            if path.is_empty() {
                return None;
            }
            Some(PnpmImpactImporter {
                path,
                dependencies: importer_dependencies(value, "dependencies"),
                dev_dependencies: importer_dependencies(value, "devDependencies"),
                optional_dependencies: importer_dependencies(value, "optionalDependencies"),
                peer_dependencies: importer_dependencies(value, "peerDependencies"),
            })
        })
        .collect();
    importers.sort_by(|a, b| a.path.cmp(&b.path));
    importers
}

fn importer_dependencies(importer: &serde_yaml::Value, field: &str) -> Vec<PnpmImporterDependency> {
    let Some(dependencies) = importer.get(field).and_then(|v| v.as_mapping()) else {
        return Vec::new();
    };
    let specifiers = importer_specifiers(importer);
    let mut result: Vec<PnpmImporterDependency> = dependencies
        .iter()
        .filter_map(|(key, value)| {
            let alias = yaml_key_to_string(key);
            if alias.is_empty() {
                return None;
            }
            Some(importer_dependency(alias, value, &specifiers))
        })
        .collect();
    result.sort_by(|a, b| {
        a.alias
            .cmp(&b.alias)
            .then(a.resolution_name.cmp(&b.resolution_name))
            .then(a.specifier.cmp(&b.specifier))
            .then(a.version.cmp(&b.version))
    });
    result
}

fn importer_specifiers(importer: &serde_yaml::Value) -> BTreeMap<String, String> {
    importer
        .get("specifiers")
        .and_then(|v| v.as_mapping())
        .into_iter()
        .flatten()
        .filter_map(|(key, value)| {
            let alias = yaml_key_to_string(key);
            let specifier = value.as_str()?.to_string();
            (!alias.is_empty()).then_some((alias, specifier))
        })
        .collect()
}

fn importer_dependency(
    alias: String,
    value: &serde_yaml::Value,
    specifiers: &BTreeMap<String, String>,
) -> PnpmImporterDependency {
    let (specifier, version) = match value {
        serde_yaml::Value::Mapping(_) => (
            value
                .get("specifier")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            value
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        ),
        serde_yaml::Value::String(version) => (
            specifiers.get(&alias).cloned().unwrap_or_default(),
            version.clone(),
        ),
        _ => (String::new(), String::new()),
    };

    PnpmImporterDependency {
        resolution_name: resolution_name_from_specifier(&specifier)
            .or_else(|| resolution_name_from_version(&version)),
        alias,
        specifier,
        version,
    }
}

fn resolution_name_from_specifier(specifier: &str) -> Option<String> {
    let stripped = specifier
        .strip_prefix("workspace:")
        .or_else(|| specifier.strip_prefix("npm:"))?;
    let aliased = stripped.strip_prefix("npm:").unwrap_or(stripped);
    let (name, _) = split_name_version(aliased);
    if !valid_package_name(name) {
        None
    } else {
        Some(name.to_string())
    }
}

fn resolution_name_from_version(version: &str) -> Option<String> {
    if !version.starts_with('/') {
        return None;
    }
    let (name, _) = split_name_version(version);
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}
