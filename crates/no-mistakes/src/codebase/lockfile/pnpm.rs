use super::{ResolutionKind, ResolvedPackage};

pub fn parse(content: &str) -> Vec<ResolvedPackage> {
    let Ok(root) = serde_yaml::from_str::<serde_yaml::Value>(content) else {
        return Vec::new();
    };
    let Some(packages_map) = root.get("packages").and_then(|v| v.as_mapping()) else {
        return Vec::new();
    };
    packages_map
        .iter()
        .map(|(key, value)| {
            let key_str = yaml_key_to_string(key);
            let (name, version) = split_name_version(&key_str);
            let (fingerprint, kind) = resolution_info(value);
            ResolvedPackage {
                name: name.to_string(),
                version: version.to_string(),
                fingerprint,
                kind,
            }
        })
        .collect()
}

fn split_name_version(key: &str) -> (&str, &str) {
    // Strip pnpm peer-dep suffix like `(yaml@2.9.0)` before splitting.
    let base = key.split_once('(').map_or(key, |(b, _)| b);
    // Strip pnpm v5/v6 leading slash (e.g. `/lodash@4.17.21` → `lodash@4.17.21`).
    let base = base.trim_start_matches('/');
    let start = usize::from(base.starts_with('@'));
    if let Some(pos) = base[start..].rfind('@') {
        (&base[..start + pos], &base[start + pos + 1..])
    } else {
        (base, "")
    }
}

fn resolution_info(value: &serde_yaml::Value) -> (String, ResolutionKind) {
    let Some(resolution) = value.get("resolution") else {
        return (String::new(), ResolutionKind::Other);
    };
    if resolution.get("repo").is_some() {
        let fp = resolution
            .get("commit")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        return (fp, ResolutionKind::Git);
    }
    if let Some(dir) = resolution.get("directory").and_then(|v| v.as_str()) {
        return (dir.to_string(), ResolutionKind::Directory);
    }
    if let Some(tarball) = resolution.get("tarball").and_then(|v| v.as_str()) {
        // Prefer integrity over tarball URL so integrity-only changes at the same URL
        // are detected as a fingerprint change.
        let integrity = resolution
            .get("integrity")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let fp = if integrity.is_empty() {
            tarball.to_string()
        } else {
            integrity.to_string()
        };
        return (fp, ResolutionKind::Tarball);
    }
    if let Some(commit) = resolution.get("commit").and_then(|v| v.as_str()) {
        return (commit.to_string(), ResolutionKind::Git);
    }
    if let Some(integrity) = resolution.get("integrity").and_then(|v| v.as_str()) {
        return (integrity.to_string(), ResolutionKind::Registry);
    }
    (String::new(), ResolutionKind::Other)
}

fn yaml_key_to_string(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests;
