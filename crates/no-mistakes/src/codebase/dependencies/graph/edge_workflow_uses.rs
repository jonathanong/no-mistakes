fn resolve_local_action_descriptor(
    root: &Path,
    target: &str,
    universe: &HashSet<PathBuf>,
    action_dirs: &[PathBuf],
) -> Option<PathBuf> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let target = target.strip_prefix("./")?;
    if !is_static_path_token(target) {
        return None;
    }
    let directory = crate::codebase::ts_resolver::normalize_path(&root.join(target));
    if !directory.starts_with(&root)
        || !action_dirs
            .iter()
            .any(|action_dir| directory.starts_with(action_dir))
    {
        return None;
    }
    ["action.yml", "action.yaml"]
        .into_iter()
        .map(|name| crate::codebase::ts_resolver::normalize_path(&directory.join(name)))
        .find(|path| universe.contains(path))
}
