pub(super) fn package_name_from_lock_key(key: &str) -> Option<String> {
    let normalized = key.strip_prefix('/').unwrap_or(key);
    if let Some(rest) = normalized.strip_prefix('@') {
        let slash = rest.find('/')?;
        let version_at = rest[slash + 1..].find('@')?;
        Some(normalized[..slash + 1 + version_at + 1].to_string())
    } else {
        let version_at = normalized.find('@')?;
        Some(normalized[..version_at].to_string())
    }
}

pub(super) fn lock_key_matches_selector(key: &str, selector: &str) -> bool {
    let normalized = key.strip_prefix('/').unwrap_or(key);
    normalized == selector
        || normalized.starts_with(&format!("{selector}("))
        || normalized.starts_with(&format!("{selector}_"))
}
