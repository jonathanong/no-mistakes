pub use no_mistakes_ts_facts::path_norm::normalize_path;

fn absolute_normalized_path(path: &Path, cwd: Option<&Path>) -> Option<PathBuf> {
    if path.is_absolute() {
        return Some(normalize_path(path));
    }
    cwd.map(|cwd| normalize_path(&cwd.join(path)))
}

/// Whether `path` stays inside `root`, comparing both against one absolute base.
/// Fails closed (returns false) when a relative path needs the current directory
/// and it cannot be read.
pub fn is_within(path: &Path, root: &Path) -> bool {
    let cwd = std::env::current_dir().ok();
    match (
        absolute_normalized_path(path, cwd.as_deref()),
        absolute_normalized_path(root, cwd.as_deref()),
    ) {
        (Some(path), Some(root)) => path.starts_with(root),
        _ => false,
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
