pub fn discover_files_preserving_roots(
    root: &Path,
    extra_skip: &[String],
    preserved_roots: &[PathBuf],
) -> Vec<PathBuf> {
    let visible_files = discover_visible_paths(root);
    discover_files_preserving_roots_from_visible(
        root,
        extra_skip,
        preserved_roots,
        &visible_files,
    )
}

pub fn discover_files_preserving_roots_from_visible(
    root: &Path,
    extra_skip: &[String],
    preserved_roots: &[PathBuf],
    visible_files: &[PathBuf],
) -> Vec<PathBuf> {
    let root = normalize_discovery_path(root);
    let mut preserved_roots: Vec<PathBuf> = preserved_roots
        .iter()
        .map(|path| normalize_discovery_path(path))
        .filter(|path| path.starts_with(&root) && path != &root)
        .collect();
    preserved_roots.sort();
    preserved_roots.dedup();

    if preserved_roots.is_empty() {
        let extra_skip: HashSet<&str> = extra_skip.iter().map(String::as_str).collect();
        return visible_files
            .iter()
            .map(|path| normalized_visible_path(path))
            .filter(|path| path.starts_with(&root))
            .filter(|path| !is_under_skipped_dir(&root, path, &extra_skip))
            .collect();
    }

    let extra_skip: HashSet<&str> = extra_skip.iter().map(String::as_str).collect();
    visible_files
        .iter()
        .map(|path| normalized_visible_path(path))
        .filter(|path| path.starts_with(&root))
        .filter(|p| {
            !is_under_skipped_dir(&root, p, &extra_skip)
                || preserved_roots
                    .iter()
                    .any(|root| p.starts_with(root) && !is_under_skipped_dir(root, p, &extra_skip))
        })
        .collect()
}
