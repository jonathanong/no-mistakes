use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn resolve_tsconfig(
    root: &Path,
    explicit: Option<&str>,
    visible_paths: &[PathBuf],
) -> Result<crate::codebase::dependencies::TsConfig> {
    let explicit = explicit.map(Path::new);
    crate::codebase::ts_resolver::resolve_tsconfig_from_visible(explicit, root, visible_paths)
}
