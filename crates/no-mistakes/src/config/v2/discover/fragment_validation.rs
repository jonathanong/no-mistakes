use super::{ConfigDocument, NoMistakesConfig};
use crate::config::source_locations::{ConfigLocations, ConfigPath};
use anyhow::Result;
use std::path::Path;

pub(super) fn validate_document(document: &ConfigDocument) -> Result<()> {
    if let Some(section) = &document.section {
        super::rule_keys::validate_fragment(&document.source, &document.path, section)?;
        super::json_relation_map_keys::validate_fragment(
            &document.source,
            &document.path,
            section,
        )?;
    } else {
        super::rule_keys::validate(&document.source, &document.path)?;
        super::json_relation_map_keys::validate(&document.source, &document.path)?;
    }
    Ok(())
}

pub(super) fn finish(
    config: &mut NoMistakesConfig,
    documents: &[ConfigDocument],
    path: &Path,
) -> Result<()> {
    super::validate_v2_config(config, path).map_err(|error| {
        super::path_globs::annotate_with_origin(error, |key| {
            let (path, local) = config.config_provenance.origin(key)?;
            let doc = documents.iter().find(|doc| doc.path == path)?;
            let line = ConfigLocations::parse(&doc.source, path)?
                .line(&local)
                .unwrap_or(1);
            Some((path.to_path_buf(), line))
        })
    })?;
    super::finish_config(config, path)?;
    for (name, policy) in &mut config.tests.vitest.projects {
        let key = ConfigPath::default()
            .key("tests")
            .key("vitest")
            .key("projects")
            .key(name)
            .key("setup_files");
        policy.setup_declaration_paths = (0..policy.setup_files.len())
            .map(|index| {
                config
                    .config_provenance
                    .origin(&key.clone().index(index))
                    .map(|(path, _)| crate::codebase::ts_resolver::normalize_path(path))
                    .unwrap_or_else(|| crate::codebase::ts_resolver::normalize_path(path))
            })
            .collect();
        if let Some(first) = policy.setup_declaration_paths.first() {
            policy.declaration_path = Some(first.clone());
        }
    }
    Ok(())
}

pub(super) fn deserialize(
    value: serde_yaml::Value,
    provenance: &super::super::schema::ConfigProvenance,
    documents: &[ConfigDocument],
) -> Result<NoMistakesConfig> {
    serde_path_to_error::deserialize(value).map_err(|error| {
        let mut key = ConfigPath::default();
        for segment in error.path() {
            key = match segment {
                serde_path_to_error::Segment::Map { key: name } => key.key(name),
                serde_path_to_error::Segment::Seq { index } => key.index(*index),
                serde_path_to_error::Segment::Enum { variant } => key.key(variant),
                serde_path_to_error::Segment::Unknown => continue,
            };
        }
        if let Some((path, local)) = provenance.origin(&key) {
            if let Some(document) = documents.iter().find(|document| document.path == path) {
                let line = ConfigLocations::parse(&document.source, path)
                    .and_then(|locations| locations.line(&local))
                    .unwrap_or(1);
                return anyhow::anyhow!("{}:{line}: {error}", path.display());
            }
        }
        anyhow::anyhow!("failed to deserialize merged configuration: {error}")
    })
}
