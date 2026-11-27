use super::super::BaseDir;
use super::types::Extracted;
use crate::codebase::rules::no_mistakes_config::paths::{self, Kind};
use crate::config::v2::NoMistakesConfig;
use std::path::Path;

pub(super) fn reports_reference(
    origin: Option<&crate::config::source_locations::ConfigPath>,
    config: &NoMistakesConfig,
    source: &Path,
) -> bool {
    // Schema paths are collected without per-field origins. Recorded
    // provenance must not drop them from the root config file.
    let declared_here = origin
        .and_then(|origin| config.config_provenance.origin(origin))
        .is_some_and(|(path, _)| path == source);
    let root_config = origin.is_none()
        && !source
            .components()
            .any(|component| component.as_os_str() == ".no-mistakes");
    declared_here || root_config
}

pub(crate) fn extract(config: &NoMistakesConfig, source: &Path) -> Vec<Extracted> {
    paths::references(config)
        .into_iter()
        .filter(|reference| reports_reference(reference.origin.as_ref(), config, source))
        .map(|reference| Extracted {
            field: reference.field,
            value: reference.value,
            allow_globs: reference.kind == Kind::Glob,
            base_dir: BaseDir::Root,
        })
        .collect()
}
