use std::path::{Path, PathBuf};

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

pub(in crate::check_discovery) fn leading_globstar_literal_prefix(
    include: &str,
) -> Option<PathBuf> {
    include.strip_prefix("**/").and_then(literal_include_prefix)
}

pub(in crate::check_discovery) fn literal_include_prefix(include: &str) -> Option<PathBuf> {
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
