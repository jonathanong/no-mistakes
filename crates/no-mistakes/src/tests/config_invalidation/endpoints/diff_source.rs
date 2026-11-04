use super::super::reconstruction::reconstruct_diff_sources;
use super::super::*;
use std::fs;

#[derive(Clone, Copy)]
pub(crate) enum DiffSide {
    Before,
    After,
}

pub(crate) struct ConfigSource {
    pub(crate) path: PathBuf,
    pub(crate) source: String,
}

#[cfg(test)]
pub(crate) fn diff_side_source(
    candidate: &Path,
    diff_files: &[DiffFile],
    side: DiffSide,
) -> Result<Option<ConfigSource>> {
    diff_side_source_with_store(candidate, diff_files, side, None)
}

pub(crate) fn diff_side_source_with_store(
    candidate: &Path,
    diff_files: &[DiffFile],
    side: DiffSide,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> Result<Option<ConfigSource>> {
    let matching = diff_files
        .iter()
        .filter(|diff| {
            same_path(&diff.path, candidate)
                || diff
                    .old_path
                    .as_ref()
                    .is_some_and(|old| same_path(old, candidate))
        })
        .collect::<Vec<_>>();
    if matching.len() > 1 {
        anyhow::bail!(
            "multiple unified diff entries match {}",
            candidate.display()
        )
    }
    let Some(diff) = matching.first() else {
        return sources
            .map_or_else(
                || fs::read_to_string(candidate),
                |sources| {
                    sources
                        .read_path(candidate)
                        .map(|source| source.to_string())
                        .map_err(|error| std::io::Error::new(error.kind(), error.to_string()))
                },
            )
            .map(|source| {
                Some(ConfigSource {
                    path: candidate.to_path_buf(),
                    source,
                })
            })
            .or_else(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(error.into())
                }
            });
    };

    let is_new_path = same_path(&diff.path, candidate);
    let is_old_path = diff
        .old_path
        .as_ref()
        .is_some_and(|old| same_path(old, candidate));
    let endpoint_matches = match (&diff.status, side) {
        (DiffFileStatus::Added, DiffSide::After) => is_new_path,
        (DiffFileStatus::Deleted, DiffSide::Before) => is_new_path,
        (DiffFileStatus::Modified, _) => is_new_path,
        (DiffFileStatus::Renamed, DiffSide::Before) => is_old_path,
        (DiffFileStatus::Renamed, DiffSide::After) => is_new_path,
        (DiffFileStatus::Copied, DiffSide::Before) => false,
        (DiffFileStatus::Copied, DiffSide::After) => is_new_path,
        _ => false,
    };
    if !endpoint_matches {
        return Ok(None);
    }
    let (before, after) = reconstruct_diff_sources(diff)?;
    let source = match side {
        DiffSide::Before => before,
        DiffSide::After => after,
    };
    Ok(source.map(|source| ConfigSource {
        path: candidate.to_path_buf(),
        source,
    }))
}
