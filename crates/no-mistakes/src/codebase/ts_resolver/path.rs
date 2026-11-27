pub use no_mistakes_ts_facts::path_norm::normalize_path;

/// Lexically normalize `path` against the current directory when it is relative,
/// so containment checks compare paths from the same base.
pub fn absolute_normalized_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return normalize_path(path);
    }
    match std::env::current_dir() {
        Ok(cwd) => normalize_path(&cwd.join(path)),
        Err(_) => normalize_path(path),
    }
}

/// Try to match `specifier` against `pattern` (which may contain a single `*`).
/// Returns `Some(capture)` where `capture` is what the `*` matched, or `""` for exact.
fn match_alias(pattern: &str, specifier: &str) -> Option<String> {
    if let Some(star) = pattern.find('*') {
        let prefix = &pattern[..star];
        let suffix = &pattern[star + 1..];
        if specifier.starts_with(prefix) && specifier.ends_with(suffix) {
            let cap_end = specifier.len() - suffix.len();
            let cap_start = prefix.len();
            return (cap_start <= cap_end).then(|| specifier[cap_start..cap_end].to_string());
        }
        None
    } else if specifier == pattern {
        Some(String::new())
    } else {
        None
    }
}
