use crate::codebase::ts_source::{FileInventory, SourceStore};
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

/// Load and validate the authored configuration, returning its merged declared values.
pub fn declared_config(root: &Path, cli_config: Option<&Path>) -> Result<serde_json::Value> {
    let visible = if cli_config.is_none() {
        crate::codebase::ts_source::discover_visible_paths(root)
    } else {
        Vec::new()
    };
    let selection = super::select_config_from_visible(root, cli_config, &visible)?;
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&visible)));
    let documents = super::selection::read_documents(&selection, |path| {
        sources
            .read_path(path)
            .map(|source| source.to_string())
            .map_err(|error| anyhow::anyhow!("reading {}: {error}", path.display()))
    })?;
    let (merged, provenance, deprecated) = super::fragments::merge_documents(root, &documents)?;
    if !documents.is_empty() {
        let mut config =
            super::fragment_validation::deserialize(merged.clone(), &provenance, &documents)?;
        config.config_provenance = provenance;
        super::aliases::mark_deprecated(&mut config, &deprecated);
        super::fragment_validation::finish(&mut config, &documents, &documents[0].path)?;
    }
    Ok(serde_json::to_value(merged)?)
}
