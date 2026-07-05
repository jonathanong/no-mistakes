use crate::codebase::ts_resolver::normalize_path;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) fn resolve_workspace_path_dependency(
    lockfile_root: &Path,
    importer: &crate::codebase::lockfile::pnpm::PnpmImporter,
    entry: &crate::codebase::lockfile::pnpm::PnpmImporterDependency,
    package_by_dir: &BTreeMap<PathBuf, String>,
) -> Option<String> {
    let path = super::super::alias::workspace_path_specifier(&entry.specifier).or_else(|| {
        entry
            .version
            .strip_prefix("link:")
            .filter(|path| relative_link_path(path))
    })?;
    let importer_dir = if super::normalize_importer_path(&importer.path) == "." {
        lockfile_root.to_path_buf()
    } else {
        lockfile_root.join(&importer.path)
    };
    let target_dir = normalize_path(&importer_dir.join(path));
    package_by_dir.get(&target_dir).cloned()
}

fn relative_link_path(path: &str) -> bool {
    !path.is_empty() && !path.starts_with('/') && !path.starts_with('\\') && !path.contains("://")
}
