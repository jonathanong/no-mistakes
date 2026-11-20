//! Package specifiers and Vite's `parent > child` include form.

pub(super) fn package_name(specifier: &str) -> Option<String> {
    let specifier = specifier.trim();
    if specifier.is_empty() || specifier.starts_with(['.', '/']) || specifier.contains([':', '\\'])
    {
        return None;
    }
    if let Some(rest) = specifier.strip_prefix('@') {
        let (scope, rest) = rest.split_once('/')?;
        let name = rest.split('/').next().unwrap_or("");
        if scope.is_empty() || name.is_empty() {
            return None;
        }
        return Some(format!("@{scope}/{name}"));
    }
    let name = specifier.split('/').next().unwrap_or("");
    (!name.is_empty()).then(|| name.to_string())
}

/// A single `parent > child` pair. Direct includes and deeper chains are not
/// this form.
pub(super) fn nested_pair(entry: &str) -> Option<(String, String)> {
    let mut parts = entry
        .split('>')
        .map(str::trim)
        .filter(|part| !part.is_empty());
    let parent = parts.next()?;
    let child = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    Some((parent.to_string(), child.to_string()))
}
