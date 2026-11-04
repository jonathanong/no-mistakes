use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::super::schema::{ConfigProvenance, NoMistakesConfig};
use super::provenance::record_provenance;
use crate::config::source_locations::ConfigPath;

#[derive(Clone, Debug)]
pub(crate) struct ConfigDocument {
    pub path: PathBuf,
    pub section: Option<String>,
    pub source: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ConfigSelection {
    pub primary: Option<PathBuf>,
    pub fragments: Vec<(PathBuf, String)>,
}

pub(crate) fn select_fragments(root: &Path, visible: &[PathBuf]) -> Result<Vec<(PathBuf, String)>> {
    let directory = root.join(".no-mistakes");
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for path in visible {
        let absolute = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        if let Some(section) = classify(root, &absolute)? {
            files.push((absolute, section));
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

fn classify(root: &Path, path: &Path) -> Result<Option<String>> {
    let directory = root.join(".no-mistakes");
    let Ok(relative) = path.strip_prefix(&directory) else {
        return Ok(None);
    };
    let parts: Vec<_> = relative.components().collect();
    if parts
        .iter()
        .any(|part| part.as_os_str().to_string_lossy().starts_with('.'))
    {
        return Ok(None);
    }
    if parts.len() < 2 {
        anyhow::bail!(
            "config file directly inside {}: {}",
            directory.display(),
            path.display()
        );
    }
    let sections = [
        "filesystem",
        "infra",
        "projects",
        "queues",
        "effects",
        "tests",
        "testPlan",
        "rules",
        "ci",
        "checks",
        "reactTraits",
    ];
    let section = parts[0].as_os_str().to_string_lossy();
    if !sections.contains(&section.as_ref()) {
        anyhow::bail!("unknown config section `{section}`: {}", path.display());
    }
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    if extension == "md" {
        return Ok(None);
    }
    if !crate::config::CONFIG_EXTENSIONS.contains(&extension) {
        anyhow::bail!("unsupported config fragment extension: {}", path.display());
    }
    Ok(Some(section.into_owned()))
}

pub(crate) fn parse_sources_quiet(
    root: &Path,
    documents: &[ConfigDocument],
) -> Result<NoMistakesConfig> {
    if documents.len() == 1 && documents[0].section.is_none() {
        return super::parse_v2_config_quiet(&documents[0].source, &documents[0].path);
    }
    for document in documents {
        let section = classify(root, &document.path)?;
        if section.is_some() && section != document.section {
            anyhow::bail!("incorrect config section for {}", document.path.display());
        }
    }
    let mut merged = Value::Mapping(Mapping::new());
    let mut origins = BTreeMap::new();
    let mut provenance = ConfigProvenance::default();
    for document in documents {
        if document.section.is_some() && classify(root, &document.path)?.is_none() {
            continue;
        }
        let value: Value = crate::config::parse_config(&document.source, &document.path)?;
        let (global, local) = if let Some(section) = &document.section {
            (ConfigPath::default().key(section), ConfigPath::default())
        } else {
            (ConfigPath::default(), ConfigPath::default())
        };
        let existing = if let Some(section) = &document.section {
            merged
                .as_mapping()
                .and_then(|map| map.get(Value::String(section.clone())))
        } else {
            Some(&merged)
        };
        record_provenance(
            &value,
            global,
            local,
            existing,
            &document.path,
            &mut provenance,
        );
        let value = if let Some(section) = &document.section {
            let mut mapping = Mapping::new();
            mapping.insert(Value::String(section.clone()), value);
            Value::Mapping(mapping)
        } else {
            value
        };
        merge(&mut merged, value, "", &document.path, &mut origins)?;
    }
    let mut config: NoMistakesConfig =
        serde_yaml::from_value(merged).context("failed to deserialize merged configuration")?;
    let path = documents.first().map(|d| d.path.as_path()).unwrap_or(root);
    config.config_provenance = provenance;
    super::finish_config(&mut config, path)?;
    for (name, policy) in &mut config.tests.vitest.projects {
        if !policy.setup_files.is_empty() {
            let key = ConfigPath::default()
                .key("tests")
                .key("vitest")
                .key("projects")
                .key(name)
                .key("setupFiles");
            if let Some((path, _)) = config.config_provenance.origin(&key) {
                policy.declaration_path = Some(crate::codebase::ts_resolver::normalize_path(path));
            }
        }
    }
    Ok(config)
}

fn merge(
    target: &mut Value,
    incoming: Value,
    key: &str,
    path: &Path,
    origins: &mut BTreeMap<String, PathBuf>,
) -> Result<()> {
    match (target, incoming) {
        (Value::Mapping(target), Value::Mapping(incoming)) => {
            for (name, value) in incoming {
                let child = format!("{key}/{}", name.as_str().unwrap_or("?"));
                if let Some(existing) = target.get_mut(&name) {
                    merge(existing, value, &child, path, origins)?;
                } else {
                    record(&value, &child, path, origins);
                    target.insert(name, value);
                }
            }
            Ok(())
        }
        (Value::Sequence(target), Value::Sequence(mut incoming)) => {
            target.append(&mut incoming);
            Ok(())
        }
        (_, _) => {
            let previous = origins
                .get(key)
                .map_or_else(|| "unknown".to_string(), |p| p.display().to_string());
            anyhow::bail!(
                "conflicting config value `{key}` in {previous} and {}",
                path.display()
            )
        }
    }
}

fn record(value: &Value, key: &str, path: &Path, origins: &mut BTreeMap<String, PathBuf>) {
    origins.insert(key.to_string(), path.to_path_buf());
    if let Value::Mapping(mapping) = value {
        for (name, value) in mapping {
            record(
                value,
                &format!("{key}/{}", name.as_str().unwrap_or("?")),
                path,
                origins,
            );
        }
    }
}
