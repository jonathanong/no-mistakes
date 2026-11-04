use super::{
    effective_v2_config_path_from_visible, emit_v2_deprecation_warnings, fragments,
    parse_sources_quiet, ConfigDocument, ConfigSelection, NoMistakesConfig,
};
use anyhow::Result;
use std::path::{Path, PathBuf};
pub(crate) fn load_v2_config_from_selected_source_store(
    root: &Path,
    selection: &ConfigSelection,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<NoMistakesConfig> {
    load_selection(root, selection, |path| {
        sources
            .read_path(path)
            .map(|source| source.to_string())
            .map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error))
    })
}

pub(super) fn load_selection(
    root: &Path,
    selection: &ConfigSelection,
    read: impl Fn(&Path) -> Result<String>,
) -> Result<NoMistakesConfig> {
    let documents = read_documents(selection, read)?;
    if documents.is_empty() {
        return Ok(NoMistakesConfig::default());
    }
    let config = parse_sources_quiet(root, &documents)?;
    emit_v2_deprecation_warnings(&config, selection.primary.as_deref().unwrap_or(root));
    Ok(config)
}

pub(crate) fn select_config_from_visible(
    root: &Path,
    cli_config: Option<&Path>,
    visible_paths: &[PathBuf],
) -> Result<ConfigSelection> {
    let primary = effective_v2_config_path_from_visible(root, cli_config, visible_paths)?;
    let fragments = if cli_config.is_none() {
        fragments::select_fragments(root, visible_paths)?
    } else {
        Vec::new()
    };
    Ok(ConfigSelection { primary, fragments })
}

pub(super) fn read_documents(
    selection: &ConfigSelection,
    read: impl Fn(&Path) -> Result<String>,
) -> Result<Vec<ConfigDocument>> {
    let mut documents = Vec::new();
    if let Some(path) = &selection.primary {
        documents.push(ConfigDocument {
            path: path.clone(),
            section: None,
            source: read(path)?,
        });
    }
    for (path, section) in &selection.fragments {
        documents.push(ConfigDocument {
            path: path.clone(),
            section: Some(section.clone()),
            source: read(path)?,
        });
    }
    Ok(documents)
}
