fn relative_path(base: &Path, path: &Path) -> Option<PathBuf> {
    let base = normalize_path(base);
    let path = normalize_path(path);
    if base.is_absolute() != path.is_absolute() {
        return None;
    }
    let base_components = base.components().collect::<Vec<_>>();
    let path_components = path.components().collect::<Vec<_>>();
    let common = base_components
        .iter()
        .zip(&path_components)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = PathBuf::new();
    for _ in common..base_components.len() {
        relative.push("..");
    }
    for component in &path_components[common..] {
        relative.push(component.as_os_str());
    }
    Some(relative)
}

fn is_config_source(path: &Path, allow_js: bool) -> bool {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("ts" | "tsx" | "mts" | "cts") => true,
        Some("js" | "jsx" | "mjs" | "cjs") => allow_js,
        _ => false,
    }
}

fn empty_config(root: &Path) -> TsConfig {
    TsConfig { dir: root.to_path_buf(), paths: Vec::new(), paths_dir: root.to_path_buf(), base_url: None }
}

fn path_depth(path: &Path) -> usize {
    path.components().count()
}

fn real_path(path: &Path) -> Option<PathBuf> {
    path.canonicalize().ok().map(|path| normalize_path(&path))
}
