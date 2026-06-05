use super::{ResolutionKind, ResolvedPackage};

pub fn parse(content: &str) -> Vec<ResolvedPackage> {
    if content.contains("__metadata:") {
        parse_berry(content)
    } else {
        parse_classic(content)
    }
}

fn parse_berry(content: &str) -> Vec<ResolvedPackage> {
    let Ok(root) = serde_yaml::from_str::<serde_yaml::Value>(content) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    let Some(map) = root.as_mapping() else {
        return Vec::new();
    };
    for (key, value) in map {
        let key_str = match key {
            serde_yaml::Value::String(s) => s.clone(),
            _ => continue,
        };
        if key_str == "__metadata" {
            continue;
        }
        let name = extract_yarn_name(&key_str);
        let version = value
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let checksum = value.get("checksum").and_then(|v| v.as_str()).unwrap_or("");
        let resolution = value
            .get("resolution")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let fingerprint = if checksum.is_empty() {
            resolution.to_string()
        } else {
            checksum.to_string()
        };
        result.push(ResolvedPackage {
            name,
            version,
            fingerprint,
            kind: ResolutionKind::Registry,
        });
    }
    result
}

fn extract_yarn_name(key: &str) -> String {
    let first = key.split(", ").next().unwrap_or(key);
    if let Some(rest) = first.strip_prefix('@') {
        if let Some(pos) = rest.find('@') {
            return first[..pos + 1].to_string();
        }
    } else if let Some(pos) = first.find('@') {
        return first[..pos].to_string();
    }
    first.to_string()
}

fn parse_classic(content: &str) -> Vec<ResolvedPackage> {
    let mut result = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_version = String::new();
    let mut current_integrity = String::new();
    let mut current_resolved = String::new();

    for line in content.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            if let Some(name) = current_name.take() {
                let fingerprint = if current_integrity.is_empty() {
                    current_resolved.clone()
                } else {
                    current_integrity.clone()
                };
                result.push(ResolvedPackage {
                    name,
                    version: current_version.clone(),
                    fingerprint,
                    kind: ResolutionKind::Registry,
                });
            }
            current_version.clear();
            current_integrity.clear();
            current_resolved.clear();
            let header = line.trim_end_matches(':').trim_matches('"');
            current_name = Some(extract_classic_name(header));
        } else {
            let trimmed = line.trim();
            if let Some(ver) = trimmed.strip_prefix("version ") {
                current_version = ver.trim_matches('"').to_string();
            } else if let Some(res) = trimmed.strip_prefix("resolved ") {
                current_resolved = res.trim_matches('"').to_string();
            } else if let Some(int) = trimmed.strip_prefix("integrity ") {
                current_integrity = int.trim_matches('"').to_string();
            }
        }
    }
    if let Some(name) = current_name {
        let fingerprint = if current_integrity.is_empty() {
            current_resolved
        } else {
            current_integrity
        };
        result.push(ResolvedPackage {
            name,
            version: current_version,
            fingerprint,
            kind: ResolutionKind::Registry,
        });
    }
    result
}

fn extract_classic_name(header: &str) -> String {
    let first = header.split(", ").next().unwrap_or(header);
    let first = first.trim_matches('"');
    if let Some(rest) = first.strip_prefix('@') {
        if let Some(pos) = rest.find('@') {
            return first[..pos + 1].to_string();
        }
    } else if let Some(pos) = first.find('@') {
        return first[..pos].to_string();
    }
    first.to_string()
}

#[cfg(test)]
mod tests;
