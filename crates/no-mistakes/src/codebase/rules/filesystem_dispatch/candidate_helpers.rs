use std::borrow::Cow;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn is_rust_path(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("rs")
}

pub(super) fn normalized_paths(paths: &[PathBuf]) -> Cow<'_, [PathBuf]> {
    let already_normalized = paths.windows(2).all(|pair| pair[0] < pair[1])
        && paths.iter().all(|path| {
            !path.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::CurDir | std::path::Component::ParentDir
                )
            })
        });
    if already_normalized {
        return Cow::Borrowed(paths);
    }
    let mut normalized = paths
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    Cow::Owned(normalized)
}

pub(super) fn markdown_inventory_path_allowed(
    request_root: &Path,
    path: &Path,
    roots: &[PathBuf],
    skip: &HashSet<&str>,
) -> bool {
    // Baselines are JSON companions rather than documentation targets, so
    // retain them for the rule's tracked-baseline validation.
    if !super::super::markdown_scope::is_mermaid_document(path) {
        return true;
    }
    !crate::codebase::ts_source::is_under_skipped_dir(request_root, path, skip)
        && roots.iter().any(|root| {
            path.starts_with(root)
                && !crate::codebase::ts_source::is_under_skipped_dir(root, path, skip)
        })
}
