use super::*;

pub(super) fn expand_globs(root: &Path, patterns: &[String]) -> Result<Vec<PathBuf>> {
    let visible_paths = no_mistakes::codebase::ts_source::discover_visible_paths(root);
    let mut files = expand_globs_from_files(root, patterns, &visible_paths)?;
    // Mirrors `WalkDir`'s default (non-follow-symlink) `file_type().is_file()`
    // check: a symlink to a file is not itself a file entry.
    files.retain(|path| std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file()));
    Ok(files)
}
