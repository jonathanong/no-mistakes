use anyhow::Result;
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
        if let Some(section) = classify_fragment(root, &absolute)? {
            files.push((absolute, section));
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

pub(crate) fn classify_fragment(root: &Path, path: &Path) -> Result<Option<String>> {
    let directory = crate::codebase::ts_resolver::normalize_path(&root.join(".no-mistakes"));
    let path = crate::codebase::ts_resolver::normalize_path(path);
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
    let section = parts[0].as_os_str().to_string_lossy();
    if !super::sections::SECTIONS.contains(&section.as_ref()) {
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
    let (merged, provenance, deprecated) = merge_documents(root, documents)?;
    let mut config: NoMistakesConfig =
        super::fragment_validation::deserialize(merged, &provenance, documents)?;
    let path = documents.first().map(|d| d.path.as_path()).unwrap_or(root);
    config.config_provenance = provenance;
    super::aliases::mark_deprecated(&mut config, &deprecated);
    super::fragment_validation::finish(&mut config, documents, path)?;
    Ok(config)
}

pub(super) fn merge_documents(
    root: &Path,
    documents: &[ConfigDocument],
) -> Result<(Value, ConfigProvenance, Vec<String>)> {
    for document in documents {
        let section = if document.section.is_some() {
            classify_fragment(root, &document.path)?
        } else {
            None
        };
        if section.is_some() && section != document.section {
            anyhow::bail!("incorrect config section for {}", document.path.display());
        }
    }
    let mut merged = Value::Mapping(Mapping::new());
    let mut origins = BTreeMap::new();
    let mut provenance = ConfigProvenance::default();
    let mut deprecated = Vec::new();
    let mut names = super::named_entries::NamedEntries::default();
    for document in documents {
        if document.section.is_some() && classify_fragment(root, &document.path)?.is_none() {
            continue;
        }
        super::fragment_validation::validate_document(document)?;
        let mut value: Value = crate::config::parse_config(&document.source, &document.path)?;
        if value.is_null() && document.section.is_none() {
            value = Value::Mapping(Mapping::new());
        }
        deprecated.extend(super::aliases::deprecated_frameworks(
            &value,
            document.section.as_deref(),
        ));
        let (global, local) = if let Some(section) = &document.section {
            (
                ConfigPath::default().key(super::aliases::canonical_key(
                    &ConfigPath::default(),
                    section,
                )),
                ConfigPath::default(),
            )
        } else {
            (ConfigPath::default(), ConfigPath::default())
        };
        let existing = if let Some(section) = &document.section {
            merged.as_mapping().and_then(|map| {
                map.get(Value::String(super::aliases::canonical_key(
                    &ConfigPath::default(),
                    section,
                )))
            })
        } else {
            Some(&merged)
        };
        record_provenance(
            &value,
            global.clone(),
            local,
            existing,
            &document.path,
            &mut provenance,
        );
        let value = super::aliases::normalize(value, &global, &document.path)?;
        let value = super::aliases::wrap_section(value, document.section.as_deref());
        names.check(&value, "", &document.path, Some(&merged))?;
        super::merge_values::merge(&mut merged, value, "", &document.path, &mut origins)?;
    }
    Ok((merged, provenance, deprecated))
}
