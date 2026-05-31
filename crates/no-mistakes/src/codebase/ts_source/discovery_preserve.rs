pub fn discover_files_preserving_roots(
    root: &Path,
    extra_skip: &[String],
    preserved_roots: &[PathBuf],
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
        return discover_files(&root, extra_skip);
    }

    match git_visible_files(&root) {
        Some(files) => discover_git_files_preserving_roots(&root, extra_skip, &preserved_roots, files),
        None => discover_walk_files_preserving_roots(&root, extra_skip, &preserved_roots),
    }
}

fn discover_git_files_preserving_roots(
    root: &Path,
    extra_skip: &[String],
    preserved_roots: &[PathBuf],
    files: Vec<String>,
) -> Vec<PathBuf> {
    let extra_skip: HashSet<&str> = extra_skip.iter().map(String::as_str).collect();
    files
        .into_iter()
        .map(|rel| normalize_discovery_path(&root.join(rel)))
        .filter(|p| p.exists())
        .filter(|p| {
            !is_under_skipped_dir(root, p, &extra_skip)
                || preserved_roots
                    .iter()
                    .any(|root| p.starts_with(root) && !is_under_skipped_dir(root, p, &extra_skip))
        })
        .collect()
}

fn discover_walk_files_preserving_roots(
    root: &Path,
    extra_skip: &[String],
    preserved_roots: &[PathBuf],
) -> Vec<PathBuf> {
    let extra_skip: HashSet<String> = extra_skip.iter().cloned().collect();
    let mut files = walk_non_ignored_files(root, &extra_skip, preserved_roots);
    files.extend(walk_github_workflow_files(root, &extra_skip));
    sort_dedup(files)
}
