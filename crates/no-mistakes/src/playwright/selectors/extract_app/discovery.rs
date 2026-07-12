//! File discovery for [`super::collect_app_selectors`].

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Candidate files under `frontend_root` that may contain app selectors.
///
/// Prefers deriving candidates from the git-visible file list (tracked files
/// plus untracked files not excluded by `.gitignore`) when `frontend_root` is
/// inside a git repository, since a raw recursive `WalkDir` walk has no
/// `.gitignore` awareness beyond [`super::super::is_skipped_dir`]'s small
/// hardcoded list and can otherwise descend into large untracked-and-ignored
/// directories (dependency stores, build output) that `git ls-files` would
/// never surface. See `crates/CLAUDE.md`'s "Never walk the tree without
/// `.gitignore` awareness". The raw walk is used only outside git
/// repositories (e.g. ad-hoc test fixtures).
pub(super) fn source_file_candidates(frontend_root: &Path) -> Vec<PathBuf> {
    use super::super::is_skipped_dir;
    match crate::codebase::ts_source::git_visible_files(frontend_root) {
        Some(files) => files
            .into_iter()
            .map(PathBuf::from)
            .filter(|rel| !rel_path_under_skipped_dir(rel))
            .map(|rel| frontend_root.join(rel))
            .filter(|path| is_file_or_symlinked_file(path))
            .collect(),
        None => WalkDir::new(frontend_root)
            .into_iter()
            .filter_entry(|entry| !is_skipped_dir(entry.path()))
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                // Use the cached WalkDir file type for regular files, but preserve
                // the previous Path::is_file behavior for symlinked source files.
                let file_type = entry.file_type();
                file_type.is_file() || file_type.is_symlink() && entry.path().is_file()
            })
            .map(|entry| entry.path().to_path_buf())
            .collect(),
    }
}

/// True if any ancestor directory component of `rel` (a path relative to the
/// discovery root, as returned by `git ls-files`) is a skip directory. This
/// mirrors [`super::super::is_skipped_dir`]'s per-directory-entry check
/// during a live filesystem walk, where descent stops at the first skip dir:
/// a match at any depth disqualifies the file. Skip directories can still be
/// git-tracked (e.g. a fixture deliberately committing a `node_modules`
/// entry), so this check must run regardless of how candidates were found.
fn rel_path_under_skipped_dir(rel: &Path) -> bool {
    use super::super::is_skipped_dir;
    rel.parent()
        .into_iter()
        .flat_map(Path::components)
        .any(|component| is_skipped_dir(Path::new(component.as_os_str())))
}

/// A regular file, or a symlink whose target resolves to a file — matching
/// the file-type check a live `WalkDir` walk performs per entry.
fn is_file_or_symlinked_file(path: &Path) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => true,
        Ok(metadata) if metadata.file_type().is_symlink() => path.is_file(),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
