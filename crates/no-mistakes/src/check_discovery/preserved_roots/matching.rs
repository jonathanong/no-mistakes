use ignore::WalkBuilder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Memoizes `git ls-files` output per discovery base directory so preserved-root
/// expansion spawns at most one `git ls-files` process per distinct base instead of
/// once per include pattern (repository-scope and every project-scope pattern reuse
/// the same entry for a given base).
pub(super) type GitFilesCache = HashMap<PathBuf, Option<Vec<String>>>;

/// Find directories under `base` whose path (relative to `base`) ends with `suffix`.
///
/// Prefers the git-visible file list (tracked files plus untracked files not excluded
/// by `.gitignore`) when `base` is inside a git repository: a preserved root only
/// matters if it contains at least one file discovery would otherwise surface, so
/// deriving candidates from that file list is both correct and avoids any filesystem
/// walk. Outside git repositories (e.g. ad-hoc test fixtures), the fallback uses
/// `ignore::WalkBuilder` so ignored dependency stores and build caches are pruned.
pub(super) fn descendant_dirs_matching_suffix(
    base: &Path,
    suffix: &Path,
    skip_directories: &[String],
    git_files_cache: &mut GitFilesCache,
) -> Vec<PathBuf> {
    let git_files = git_files_cache
        .entry(base.to_path_buf())
        .or_insert_with(|| no_mistakes::codebase::ts_source::git_visible_files(base));
    match git_files {
        Some(files) => descendant_dirs_matching_suffix_from_files(
            base,
            suffix,
            files.as_slice(),
            skip_directories,
        ),
        None => {
            let mut roots = Vec::new();
            collect_descendant_dirs_matching_suffix(
                base,
                base,
                suffix,
                skip_directories,
                &mut roots,
            );
            roots
        }
    }
}

/// Same matching + skip-descent semantics as the ignore-aware fallback,
/// but walks each git-visible file's directory components top-down instead of the
/// filesystem: a directory is checked against `suffix` and then, if its name is
/// skip-listed, no directory nested beneath it is considered (mirroring the fallback's
/// "check, then don't descend" order) — without any filesystem walk.
pub(super) fn descendant_dirs_matching_suffix_from_files(
    base: &Path,
    suffix: &Path,
    files: &[String],
    skip_directories: &[String],
) -> Vec<PathBuf> {
    descendant_dirs_matching_suffix_from_paths(
        base,
        suffix,
        files.iter().map(Path::new),
        skip_directories,
    )
}

pub(super) fn descendant_dirs_matching_suffix_from_paths<'a>(
    base: &Path,
    suffix: &Path,
    files: impl Iterator<Item = &'a Path>,
    skip_directories: &[String],
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for rel in files {
        // `rel` is already relative to `base` (git ls-files output), so the directory
        // chain is built purely lexically here and only joined onto `base` once a
        // match is found — no `strip_prefix` round-trip needed.
        let rel_dir = Path::new(rel).parent().unwrap_or_else(|| Path::new(""));
        let mut accumulated = PathBuf::new();
        for component in rel_dir.components() {
            accumulated.push(component);
            if accumulated.ends_with(suffix) {
                roots.push(base.join(&accumulated));
            }
            let name = component.as_os_str().to_str().unwrap_or_default();
            let skipped = no_mistakes::codebase::ts_source::is_skipped_dir(name)
                || skip_directories.iter().any(|skip| skip == name);
            if skipped {
                break;
            }
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

pub(super) fn collect_descendant_dirs_matching_suffix(
    base: &Path,
    dir: &Path,
    suffix: &Path,
    skip_directories: &[String],
    roots: &mut Vec<PathBuf>,
) {
    let base = base.to_path_buf();
    let suffix = suffix.to_path_buf();
    let skip_directories = skip_directories.to_vec();
    let matches = Arc::new(Mutex::new(Vec::new()));
    let matches_for_filter = Arc::clone(&matches);
    let filter_base = base.clone();
    let filter_suffix = suffix.clone();

    let mut builder = WalkBuilder::new(dir);
    builder
        .hidden(true)
        .require_git(false)
        .filter_entry(move |entry| {
            if entry.depth() == 0
                || !entry
                    .file_type()
                    .is_some_and(|file_type| file_type.is_dir())
            {
                return true;
            }
            let path = entry.path();
            if path
                .strip_prefix(&filter_base)
                .ok()
                .is_some_and(|rel| rel.ends_with(&filter_suffix))
            {
                matches_for_filter
                    .lock()
                    .expect("preserved-root match lock should not be poisoned")
                    .push(path.to_path_buf());
            }
            let name = entry.file_name().to_str().unwrap_or_default();
            !no_mistakes::codebase::ts_source::is_skipped_dir(name)
                && !skip_directories.iter().any(|skip| skip == name)
        });
    for _ in builder.build() {}
    drop(builder);

    let mut matches = Arc::try_unwrap(matches)
        .expect("preserved-root walker should release its match collector")
        .into_inner()
        .expect("preserved-root match lock should not be poisoned");
    matches.sort();
    matches.dedup();
    roots.extend(matches);
}

pub(super) fn leading_globstar_literal_prefix(include: &str) -> Option<PathBuf> {
    include.strip_prefix("**/").and_then(literal_include_prefix)
}

pub(super) fn literal_include_prefix(include: &str) -> Option<PathBuf> {
    let prefix = include
        .split(['*', '?', '[', '{'])
        .next()
        .unwrap_or_default()
        .trim_end_matches('/');
    if prefix.is_empty() {
        return None;
    }
    Some(PathBuf::from(prefix))
}
