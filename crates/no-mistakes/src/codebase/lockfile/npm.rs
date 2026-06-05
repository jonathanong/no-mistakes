use super::{ResolutionKind, ResolvedPackage};

pub fn parse(content: &str) -> Vec<ResolvedPackage> {
    let Ok(root) = serde_json::from_str::<serde_json::Value>(content) else {
        return Vec::new();
    };
    let version = root
        .get("lockfileVersion")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    if version >= 2 {
        parse_v2(&root)
    } else {
        parse_v1(&root)
    }
}

fn parse_v2(root: &serde_json::Value) -> Vec<ResolvedPackage> {
    let Some(packages) = root.get("packages").and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    packages
        .iter()
        .filter(|(k, _)| !k.is_empty())
        .map(|(key, value)| {
            let name = key
                .trim_start_matches("node_modules/")
                .split("/node_modules/")
                .last()
                .unwrap_or(key.as_str());
            let version = value
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let integrity = value
                .get("integrity")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let resolved = value.get("resolved").and_then(|v| v.as_str()).unwrap_or("");
            let kind = if value.get("link").and_then(|v| v.as_bool()).unwrap_or(false) {
                ResolutionKind::Workspace
            } else if resolved.is_empty() {
                ResolutionKind::Directory
            } else {
                ResolutionKind::Registry
            };
            let fingerprint = if integrity.is_empty() {
                resolved.to_string()
            } else {
                integrity.to_string()
            };
            ResolvedPackage {
                name: name.to_string(),
                version,
                fingerprint,
                kind,
            }
        })
        .collect()
}

fn parse_v1(root: &serde_json::Value) -> Vec<ResolvedPackage> {
    let Some(deps) = root.get("dependencies").and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    collect_v1_deps(deps)
}

fn collect_v1_deps(deps: &serde_json::Map<String, serde_json::Value>) -> Vec<ResolvedPackage> {
    let mut result = Vec::new();
    for (name, value) in deps {
        let version = value
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let integrity = value
            .get("integrity")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let resolved = value.get("resolved").and_then(|v| v.as_str()).unwrap_or("");
        let fingerprint = if integrity.is_empty() {
            resolved.to_string()
        } else {
            integrity.to_string()
        };
        result.push(ResolvedPackage {
            name: name.clone(),
            version,
            fingerprint,
            kind: ResolutionKind::Registry,
        });
        if let Some(nested) = value.get("dependencies").and_then(|v| v.as_object()) {
            result.extend(collect_v1_deps(nested));
        }
    }
    result
}

#[cfg(test)]
mod tests;
