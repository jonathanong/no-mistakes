fn try_resolve(path: &Path) -> Option<PathBuf> {
    if path.exists() {
        return Some(path.to_path_buf());
    }
    // Try appending TS extensions if no extension present.
    let s = path.to_string_lossy();
    for ext in &[".mts", ".ts", ".tsx", ".mjs", ".js", ".jsx"] {
        let candidate = PathBuf::from(format!("{s}{ext}"));
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

