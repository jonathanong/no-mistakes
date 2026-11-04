use super::*;

pub(super) fn paths_change_config(args: &PlanArgs, root: &Path, paths: &[PathBuf]) -> bool {
    paths.iter().any(|path| is_config_input(args, root, path))
}

pub(super) fn manual_config_paths_are_reconstructed(
    args: &PlanArgs,
    root: &Path,
    collected: &ChangedFiles,
) -> bool {
    collected
        .manual_files
        .iter()
        .filter(|path| is_config_input(args, root, path))
        .all(|manual| {
            collected
                .diff_files
                .iter()
                .any(|diff| diff_mentions_path(diff, manual))
        })
}

pub(super) fn diff_changes_config(args: &PlanArgs, root: &Path, diff_files: &[DiffFile]) -> bool {
    diff_files.iter().any(|diff| {
        is_config_input(args, root, &diff.path)
            || diff
                .old_path
                .as_ref()
                .is_some_and(|path| is_config_input(args, root, path))
    })
}

fn diff_mentions_path(diff: &DiffFile, path: &Path) -> bool {
    same_path(&diff.path, path)
        || diff
            .old_path
            .as_ref()
            .is_some_and(|old| same_path(old, path))
}

fn is_config_input(args: &PlanArgs, root: &Path, path: &Path) -> bool {
    args.config.as_deref().map_or_else(
        || automatic_input(root, path),
        |config| same_path(&normalize(root, config), &normalize(root, path)),
    )
}

pub(crate) fn automatic_input(root: &Path, path: &Path) -> bool {
    let path = normalize(root, path);
    (path.starts_with(normalize(root, &root.join(".no-mistakes")))
        && crate::config::v2::discover::classify_fragment(root, &path)
            .map_or(true, |section| section.is_some()))
        || automatic_config_candidates(root)
            .iter()
            .any(|candidate| same_path(candidate, &path))
}

pub(super) fn automatic_config_candidates(root: &Path) -> Vec<PathBuf> {
    crate::config::v2::discover::automatic_v2_config_paths(root)
        .into_iter()
        .map(|path| normalize(root, &path))
        .collect()
}

pub(crate) fn changed_config_path(
    args: &PlanArgs,
    root: &Path,
    collected: &ChangedFiles,
) -> Option<PathBuf> {
    collected
        .files
        .iter()
        .find(|path| is_config_input(args, root, path))
        .cloned()
        .or_else(|| {
            collected.diff_files.iter().find_map(|diff| {
                std::iter::once(&diff.path)
                    .chain(diff.old_path.iter())
                    .find(|path| is_config_input(args, root, path))
                    .cloned()
            })
        })
}
