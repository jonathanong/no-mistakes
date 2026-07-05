use super::{split_name_version, yaml_key_to_string};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PnpmImporter {
    pub path: String,
    pub dependencies: Vec<PnpmImporterDependency>,
    pub dev_dependencies: Vec<PnpmImporterDependency>,
    pub optional_dependencies: Vec<PnpmImporterDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PnpmImporterDependency {
    pub alias: String,
    pub resolution_name: Option<String>,
    pub specifier: String,
    pub version: String,
}

pub fn parse_importers(content: &str) -> Vec<PnpmImporter> {
    let Ok(root) = serde_yaml::from_str::<serde_yaml::Value>(content) else {
        return Vec::new();
    };
    let Some(importers_map) = root.get("importers").and_then(|v| v.as_mapping()) else {
        return Vec::new();
    };

    let mut importers: Vec<PnpmImporter> = importers_map
        .iter()
        .filter_map(|(key, value)| {
            let path = yaml_key_to_string(key);
            if path.is_empty() {
                return None;
            }
            Some(PnpmImporter {
                path,
                dependencies: importer_dependencies(value, "dependencies"),
                dev_dependencies: importer_dependencies(value, "devDependencies"),
                optional_dependencies: importer_dependencies(value, "optionalDependencies"),
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
    let mut result: Vec<PnpmImporterDependency> = dependencies
        .iter()
        .filter_map(|(key, value)| {
            let alias = yaml_key_to_string(key);
            if alias.is_empty() {
                return None;
            }
            Some(importer_dependency(alias, value))
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

fn importer_dependency(alias: String, value: &serde_yaml::Value) -> PnpmImporterDependency {
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
        serde_yaml::Value::String(version) => (String::new(), version.clone()),
        _ => (String::new(), String::new()),
    };

    PnpmImporterDependency {
        resolution_name: resolution_name_from_specifier(&specifier),
        alias,
        specifier,
        version,
    }
}

fn resolution_name_from_specifier(specifier: &str) -> Option<String> {
    let aliased = specifier.strip_prefix("npm:")?;
    let (name, _) = split_name_version(aliased);
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}
