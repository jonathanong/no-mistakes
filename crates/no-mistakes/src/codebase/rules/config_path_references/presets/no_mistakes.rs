use super::super::BaseDir;
use super::types::Extracted;
use crate::codebase::rules::no_mistakes_config::paths::{self, Kind};
use crate::config::v2::NoMistakesConfig;
use std::path::Path;

pub(crate) fn extract(config: &NoMistakesConfig, source: &Path) -> Vec<Extracted> {
    paths::references(config)
        .into_iter()
        .filter(|reference| {
            reference
                .origin
                .as_ref()
                .and_then(|origin| config.config_provenance.origin(origin))
                .is_some_and(|(path, _)| path == source)
                || (config.config_provenance.is_empty()
                    && !source
                        .components()
                        .any(|component| component.as_os_str() == ".no-mistakes"))
        })
        .map(|reference| Extracted {
            field: reference.field,
            value: reference.value,
            allow_globs: reference.kind == Kind::Glob,
            base_dir: BaseDir::Root,
        })
        .collect()
}
