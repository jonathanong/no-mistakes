fn resolve_tsconfig_from_visible(
    root: &Path,
    explicit: Option<&Path>,
    visible_paths: &[PathBuf],
) -> Result<TsConfig> {
    let explicit_path = explicit.is_some();
    let path = match explicit {
        Some(path) if path.is_absolute() => Some(path.to_path_buf()),
        Some(path) => Some(root.join(path)),
        None => find_tsconfig_from_visible(root, visible_paths),
    };
    match path {
        Some(path) if explicit_path => {
            load_tsconfig(&path).context(format!("loading tsconfig {}", path.display()))
        }
        Some(path) => Ok(load_tsconfig(&path).unwrap_or_else(|_| empty_tsconfig(root))),
        None => Ok(empty_tsconfig(root)),
    }
}

fn empty_tsconfig(root: &Path) -> TsConfig {
    TsConfig {
        dir: root.to_path_buf(),
        paths_dir: root.to_path_buf(),
        paths: Vec::new(),
        base_url: None,
    }
}
