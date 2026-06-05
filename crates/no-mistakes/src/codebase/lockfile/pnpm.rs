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
    // Strip peer-dep suffix enclosed in parens: pkg@ver(peer@X) → pkg@ver
    let base = key.split_once('(').map_or(key, |(b, _)| b);
    // Strip pnpm v5/v6 leading slash (e.g. `/lodash@4.17.21` → `lodash@4.17.21`).
    // Track whether the key had a leading slash: only v5 slash-separated keys do,
    // so we use this to distinguish `/lodash/4.17.21` from `github.com/org/repo`.
    let had_leading_slash = base.starts_with('/');
    let base = base.trim_start_matches('/');

    if let Some(stripped) = base.strip_prefix('@') {
        // Scoped package. suffix = "scope/pkg[@ver | /ver]..."
        let Some(first_slash) = stripped.find('/') else {
            return (base, "");
        };
        let first_slash = first_slash + 1; // adjust past leading '@'
        let pkg_rest = &base[first_slash + 1..]; // "pkg@ver" (v6/v7) or "pkg/ver" (v5)
        let at_in_pkg = pkg_rest.find('@');
        let slash_in_pkg = pkg_rest.find('/');
        match (at_in_pkg, slash_in_pkg) {
            (Some(a), Some(s)) if s < a => {
                // v5 scoped with peer suffix: @scope/pkg/ver_peer@ver
                let ver_raw = &pkg_rest[s + 1..];
                (
                    &base[..first_slash + 1 + s],
                    ver_raw.split_once('_').map_or(ver_raw, |(v, _)| v),
                )
            }
            (Some(a), _) => {
                // v6/v7 scoped: @scope/pkg@ver[_peer]
                let ver_raw = &pkg_rest[a + 1..];
                (
                    &base[..first_slash + 1 + a],
                    ver_raw.split_once('_').map_or(ver_raw, |(v, _)| v),
                )
            }
            (None, Some(s)) => {
                // v5 scoped without peer suffix: @scope/pkg/ver
                let ver_raw = &pkg_rest[s + 1..];
                (
                    &base[..first_slash + 1 + s],
                    ver_raw.split_once('_').map_or(ver_raw, |(v, _)| v),
                )
            }
            (None, None) => (base, ""),
        }
    } else {
        // Unscoped. Prefer the first '@' (version sep) over '/' (v5 sep) unless '/' comes first.
        // Only treat '/' as a version separator when the original key had a leading slash;
        // without it, slashes are part of the name (e.g. `github.com/org/repo`).
        let first_at = base.find('@');
        let first_slash = base.find('/');
        match (first_at, first_slash) {
            (Some(a), Some(s)) if had_leading_slash && s < a => {
                // v5 unscoped with peer suffix: /pkg/ver_peer@ver
                let ver_raw = &base[s + 1..];
                (
                    &base[..s],
                    ver_raw.split_once('_').map_or(ver_raw, |(v, _)| v),
                )
            }
            (Some(a), _) => {
                // v6/v7 unscoped: pkg@ver[_peer]
                let ver_raw = &base[a + 1..];
                (
                    &base[..a],
                    ver_raw.split_once('_').map_or(ver_raw, |(v, _)| v),
                )
            }
            (None, Some(s)) if had_leading_slash => {
                // v5 unscoped without peer suffix: /pkg/ver
                let ver_raw = &base[s + 1..];
                (
                    &base[..s],
                    ver_raw.split_once('_').map_or(ver_raw, |(v, _)| v),
                )
            }
            _ => (base, ""),
        }
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
