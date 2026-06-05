use super::{ResolutionKind, ResolvedPackage};

pub fn parse(content: &str) -> Vec<ResolvedPackage> {
    let Ok(root) = serde_json::from_str::<serde_json::Value>(content) else {
        return Vec::new();
    };
    let Some(packages) = root.get("packages").and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    packages
        .iter()
        .filter_map(|(name, entry)| {
            let arr = entry.as_array()?;
            let specifier = arr.first().and_then(|v| v.as_str()).unwrap_or("");
            let version = specifier
                .rsplit_once('@')
                .map(|(_, v)| v)
                .unwrap_or("");
            let info = arr.get(2);
            let integrity = info
                .and_then(|v| v.get("integrity"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let resolved = info
                .and_then(|v| v.get("resolved"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let fingerprint = if integrity.is_empty() {
                resolved.to_string()
            } else {
                integrity.to_string()
            };
            Some(ResolvedPackage {
                name: name.clone(),
                version: version.to_string(),
                fingerprint,
                kind: ResolutionKind::Registry,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
