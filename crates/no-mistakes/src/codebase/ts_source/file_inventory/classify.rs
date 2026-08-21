use super::{ClassifiedPath, FileClassification};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitIndexKind {
    RegularFile,
    Symlink,
}

/// Classify already-discovered paths with parallel `symlink_metadata` calls.
/// Callers that already have Git index modes should use
/// [`classify_git_listed_paths`] so tracked regular files skip these syscalls.
pub(super) fn inventory_paths(paths: &[PathBuf]) -> (Vec<ClassifiedPath>, usize) {
    crate::perf_trace::trace("discovery.inventory_classify", || {
        let classified: Vec<ClassifiedPath> = paths
            .par_iter()
            .filter_map(|path| {
                crate::invocation::check_timeout().ok()?;
                Some(stat_inventory_path(super::super::normalize_discovery_path(
                    path,
                )))
            })
            .collect();
        let metadata_stats = classified.len();
        (classified, metadata_stats)
    })
}

/// Classify Git-listed relative paths. Tracked regular files (`100644`/`100755`)
/// use index mode and skip worktree metadata; tracked symlinks (`120000`) still
/// call `Path::is_file`. Untracked paths and other modes keep `symlink_metadata`.
///
/// Skipping stats for `100644` trusts the index: an unstaged file→symlink swap
/// is still classified as a regular file. Missing worktree paths are dropped
/// from the `R` records produced by `git ls-files --deleted`, not by statting.
pub(crate) fn classify_git_listed_paths(
    root: &Path,
    paths: Vec<(PathBuf, Option<GitIndexKind>)>,
) -> (Vec<ClassifiedPath>, usize) {
    crate::perf_trace::trace("discovery.classify", || {
        let classified: Vec<(ClassifiedPath, bool)> = paths
            .into_par_iter()
            .filter_map(|(relative, index_kind)| {
                crate::invocation::check_timeout().ok()?;
                classify_listed_path(root.join(relative), index_kind)
            })
            .collect();
        let metadata_stats = classified.iter().filter(|(_, stated)| *stated).count();
        (
            classified.into_iter().map(|(entry, _)| entry).collect(),
            metadata_stats,
        )
    })
}

fn classify_listed_path(
    path: PathBuf,
    index_kind: Option<GitIndexKind>,
) -> Option<(ClassifiedPath, bool)> {
    match index_kind {
        Some(GitIndexKind::RegularFile) => Some((
            ClassifiedPath {
                path,
                classification: FileClassification::TRACKED_REGULAR,
            },
            false,
        )),
        Some(GitIndexKind::Symlink) => Some((
            ClassifiedPath {
                classification: FileClassification::from_tracked_symlink(&path),
                path,
            },
            true,
        )),
        None => stat_existing_path(path).map(|entry| (entry, true)),
    }
}

fn stat_inventory_path(path: PathBuf) -> ClassifiedPath {
    let classification = std::fs::symlink_metadata(&path)
        .ok()
        .map_or_else(FileClassification::default, |metadata| {
            FileClassification::from_file_type(&path, metadata.file_type())
        });
    ClassifiedPath {
        path,
        classification,
    }
}

fn stat_existing_path(path: PathBuf) -> Option<ClassifiedPath> {
    let metadata = std::fs::symlink_metadata(&path).ok()?;
    Some(ClassifiedPath {
        classification: FileClassification::from_file_type(&path, metadata.file_type()),
        path,
    })
}
