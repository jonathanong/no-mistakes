use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use std::path::{Path, PathBuf};

pub(in crate::codebase::rules::forbidden_workspace_closure) fn base_root<'a>(
    root: &'a Path,
    target_roots: &'a [PathBuf],
) -> &'a Path {
    target_roots
        .first()
        .filter(|_| target_roots.len() == 1)
        .map_or(root, PathBuf::as_path)
}

pub(super) fn absolute_lockfile_path(
    root: &Path,
    lockfile_base: &Path,
    lockfile: &Path,
) -> PathBuf {
    let path = if lockfile.is_absolute() {
        lockfile.to_path_buf()
    } else if lockfile_base != root {
        let project_lockfile = lockfile_base.join(lockfile);
        if project_lockfile.exists() {
            project_lockfile
        } else {
            root.join(lockfile)
        }
    } else {
        root.join(lockfile)
    };
    normalize_path(&path)
}

pub(super) fn importer_key(lockfile_root: &Path, package_dir: &Path) -> String {
    let rel_dir = relative_slash_path(lockfile_root, package_dir);
    if rel_dir.is_empty() {
        ".".to_string()
    } else {
        rel_dir
    }
}

pub(in crate::codebase::rules::forbidden_workspace_closure) fn normalize_importer_path(
    path: &str,
) -> String {
    let normalized = path.trim_start_matches("./").trim_end_matches('/');
    if normalized.is_empty() {
        ".".to_string()
    } else {
        normalized.to_string()
    }
}
