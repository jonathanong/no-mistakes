use crate::codebase::ts_resolver::ImportResolver;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) use crate::codebase::ts_resolver::TsConfig;

pub(crate) fn load_tsconfig_from_visible(
    root: &Path,
    explicit: Option<&Path>,
    visible_paths: &[PathBuf],
) -> Result<TsConfig> {
    crate::codebase::ts_resolver::resolve_tsconfig_from_visible(explicit, root, visible_paths)
}

pub(crate) fn queue_import_resolver<'a>(
    tsconfig: &'a TsConfig,
    root: &'a Path,
    visible_files: &'a HashSet<PathBuf>,
) -> ImportResolver<'a> {
    ImportResolver::new(tsconfig)
        .with_queue_compatibility(root)
        .with_visible(visible_files)
}
