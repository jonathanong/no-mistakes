use super::super::preserved_roots::{leading_globstar_literal_prefix, literal_include_prefix};
use ignore::{DirEntry, WalkBuilder};
use std::path::{Path, PathBuf};

/// Walks one known-non-Git base exactly once, reopening built-in skipped
/// directories only when an explicit include can preserve their contents.
pub(in crate::check_discovery) fn walk_ignore_aware_universe(
    root: &Path,
    includes: &[String],
    reopened_roots: &[PathBuf],
    reopened_suffixes: &[PathBuf],
) -> Vec<PathBuf> {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(root);
    let literal_roots: Vec<_> = includes
        .iter()
        .filter_map(|include| literal_include_prefix(include))
        .map(|prefix| no_mistakes::codebase::ts_resolver::normalize_path(&root.join(prefix)))
        .collect();
    let suffixes: Vec<_> = includes
        .iter()
        .filter_map(|include| leading_globstar_literal_prefix(include))
        .collect();
    let filter_root = root.clone();
    let reopened_roots = reopened_roots.to_vec();
    let reopened_suffixes = reopened_suffixes.to_vec();
    let mut builder = WalkBuilder::new(&root);
    builder
        .hidden(false)
        .require_git(false)
        .filter_entry(move |entry| {
            visible_entry(
                entry,
                &filter_root,
                &literal_roots,
                &suffixes,
                &reopened_roots,
                &reopened_suffixes,
            )
        });

    let mut files: Vec<_> = builder
        .build()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .is_some_and(|file_type| file_type.is_file())
        })
        .map(|entry| no_mistakes::codebase::ts_resolver::normalize_path(entry.path()))
        .collect();
    files.sort();
    files.dedup();
    files
}

fn visible_entry(
    entry: &DirEntry,
    root: &Path,
    literal_roots: &[PathBuf],
    suffixes: &[PathBuf],
    reopened_roots: &[PathBuf],
    reopened_suffixes: &[PathBuf],
) -> bool {
    if entry.depth() == 0 {
        return true;
    }
    let path = entry.path();
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    if !visible_hidden_path(relative) {
        return false;
    }
    if !entry
        .file_type()
        .is_some_and(|file_type| file_type.is_dir())
    {
        return true;
    }
    let name = entry.file_name().to_str().unwrap_or_default();
    if !no_mistakes::codebase::ts_source::is_skipped_dir(name) {
        return true;
    }
    literal_roots
        .iter()
        .any(|preserved_root| preserved_root.starts_with(path))
        || reopened_roots
            .iter()
            .any(|reopened_root| reopened_root.starts_with(path))
        || reopened_suffixes
            .iter()
            .any(|suffix| relative.ends_with(suffix))
        || suffixes.iter().any(|suffix| relative.ends_with(suffix))
}

fn visible_hidden_path(relative: &Path) -> bool {
    let mut components = relative
        .components()
        .filter_map(|component| component.as_os_str().to_str());
    let Some(first) = components.next() else {
        return true;
    };
    if first == ".github" {
        return matches!(components.next(), None | Some("workflows"));
    }
    !first.starts_with('.') && !components.any(|name| name.starts_with('.'))
}
