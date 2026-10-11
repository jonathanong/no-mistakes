use super::config;
use crate::codebase::dependencies::graph::GraphFiles;
use crate::codebase::ts_resolver::{normalize_path, ImportResolution};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[cfg(test)]
#[path = "standalone/tests/imported_helpers.rs"]
mod imported_helpers;
#[cfg(test)]
#[path = "standalone/tests/legacy.rs"]
pub(super) mod legacy;
#[cfg(test)]
#[path = "standalone/tests/setup_mocks.rs"]
mod setup_mocks;

pub(crate) fn resolve_mock_specifiers(
    specifiers: &[String],
    file: &Path,
    resolver: &dyn ImportResolution,
    graph_files: Option<&GraphFiles>,
) -> HashSet<PathBuf> {
    specifiers
        .iter()
        .map(|specifier| {
            resolver
                .resolve(specifier, file)
                .map(|path| remap_resolved_path(graph_files, path))
                .unwrap_or_else(|| PathBuf::from(specifier))
        })
        .collect()
}

/// Keep resolver results in the lexical namespace owned by the dependency
/// graph. A scoped resolver may return a canonical target through a symlinked
/// package root, while graph nodes and discovered manual mocks retain the
/// caller-visible lexical path.
pub(crate) fn remap_resolved_path(graph_files: Option<&GraphFiles>, path: PathBuf) -> PathBuf {
    graph_files
        .and_then(|files| files.visible_path(&path).map(Path::to_path_buf))
        .unwrap_or(path)
}

pub(crate) fn matching_test_files_with_filter(
    root: &Path,
    files: &[PathBuf],
    filter: &config::TestFilter,
) -> Vec<PathBuf> {
    files
        .iter()
        .filter(|file| crate::codebase::dependencies::extract::is_indexable(file))
        .filter(|file| {
            filter.is_match(&crate::codebase::ts_source::relative_slash_path(root, file))
        })
        .map(|file| normalize_path(file))
        .collect()
}
