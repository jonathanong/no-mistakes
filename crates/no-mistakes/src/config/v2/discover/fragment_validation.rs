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
        if !policy.setup_files.is_empty() {
            let key = ConfigPath::default()
                .key("tests")
                .key("vitest")
                .key("projects")
                .key(name)
                .key("setup_files")
                .index(0);
            if let Some((path, _)) = config.config_provenance.origin(&key) {
                policy.declaration_path = Some(crate::codebase::ts_resolver::normalize_path(path));
            }
        }
    }
    Ok(())
}
