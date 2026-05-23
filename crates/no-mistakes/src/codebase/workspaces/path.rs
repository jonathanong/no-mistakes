fn try_resolve(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.to_path_buf());
    }
    // Try appending TS extensions if no extension present.
    let s = path.to_string_lossy();
    for ext in &[".mts", ".ts", ".tsx", ".mjs", ".js", ".jsx"] {
        let candidate = PathBuf::from(format!("{s}{ext}"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    if path.is_dir() {
        for name in &[
            "index.mts",
            "index.ts",
            "index.tsx",
            "index.mjs",
            "index.js",
            "index.jsx",
        ] {
            let candidate = path.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}
