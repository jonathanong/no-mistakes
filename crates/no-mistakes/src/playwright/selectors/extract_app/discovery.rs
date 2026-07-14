//! File discovery for [`super::collect_app_selectors`].

use std::path::{Path, PathBuf};

/// Candidate files under `frontend_root` that may contain app selectors.
///
/// Uses the shared ignore-aware candidate list, then applies selector-specific
/// skip-directory and symlink policies.
pub(super) fn source_file_candidates(frontend_root: &Path) -> Vec<PathBuf> {
    crate::codebase::ts_source::discover_visible_classified_paths(frontend_root)
        .into_iter()
        .filter(|entry| {
            entry
                .path
                .strip_prefix(frontend_root)
                .is_ok_and(|rel| !rel_path_under_skipped_dir(rel))
        })
        .filter(|entry| entry.classification.target_is_file())
        .map(|entry| entry.path)
        .collect()
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
#[cfg(test)]
mod tests;
