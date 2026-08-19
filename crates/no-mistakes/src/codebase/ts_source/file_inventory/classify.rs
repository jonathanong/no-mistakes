use super::{ClassifiedPath, FileClassification};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

/// Classify already-discovered paths with parallel `symlink_metadata` calls.
/// Git listing does not report file-vs-symlink type, so the syscalls stay;
/// they do not need to run on one thread. Timeout checks are per path because
/// a parallel `take_while` would drop in-flight successes.
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

pub(crate) fn classify_relative_paths(root: &Path, paths: Vec<PathBuf>) -> Vec<ClassifiedPath> {
    crate::perf_trace::trace("discovery.classify", || {
        paths
            .into_par_iter()
            .filter_map(|relative| {
                crate::invocation::check_timeout().ok()?;
                stat_existing_path(root.join(relative))
            })
            .collect()
    })
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
