use super::filter_source_files;
use std::path::{Path, PathBuf};

pub(super) fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(super) fn shared_symbol_files(
    workspace_files: &[PathBuf],
    analysis_files: &[PathBuf],
) -> Vec<PathBuf> {
    let mut symbol_files = workspace_files.to_vec();
    symbol_files.extend(analysis_files.iter().cloned());
    symbol_files.sort();
    symbol_files.dedup();
    filter_source_files(&symbol_files)
}
