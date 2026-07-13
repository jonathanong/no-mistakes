fn try_resolve(path: &Path) -> Option<PathBuf> {
    try_resolve_inner(path, None)
}

fn try_resolve_from_visible(
    path: &Path,
    visible_files: &std::collections::HashSet<PathBuf>,
) -> Option<PathBuf> {
    try_resolve_inner(path, Some(visible_files))
}

fn try_resolve_inner(
    path: &Path,
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> Option<PathBuf> {
    let path = normalize_path(path);
    if workspace_path_is_file(&path, visible_files) {
        return Some(path);
    }
    // Try appending TS extensions if no extension present.
    let s = path.to_string_lossy();
    for ext in &[".mts", ".ts", ".tsx", ".mjs", ".js", ".jsx"] {
        let candidate = PathBuf::from(format!("{s}{ext}"));
        if workspace_path_is_file(&candidate, visible_files) {
            return Some(candidate);
        }
    }
    if visible_files.is_some() || path.is_dir() {
        for name in &[
            "index.mts",
            "index.ts",
            "index.tsx",
            "index.mjs",
            "index.js",
            "index.jsx",
        ] {
            let candidate = path.join(name);
            if workspace_path_is_file(&candidate, visible_files) {
                return Some(candidate);
            }
        }
    }
    None
}

fn workspace_path_is_file(
    path: &Path,
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> bool {
    visible_files.map_or_else(
        || path.is_file(),
        |visible| visible.contains(&normalize_path(path)),
    )
}
