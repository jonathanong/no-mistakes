use super::{ResolutionKind, ResolvedPackage};

mod classic;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
use classic::extract_classic_name;
use classic::parse_classic_inner;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum YarnValidationError {
    Malformed,
}

pub fn parse(content: &str) -> Vec<ResolvedPackage> {
    if content.contains("__metadata:") {
        parse_berry_inner(content, false).unwrap_or_default()
    } else {
        parse_classic_inner(content, false).unwrap_or_default()
    }
}

pub(crate) fn parse_checked(content: &str) -> Result<Vec<ResolvedPackage>, YarnValidationError> {
    if content.contains("__metadata:") {
        parse_berry_inner(content, true)
    } else {
        parse_classic_inner(content, true)
    }
}

fn parse_berry_inner(
    content: &str,
    strict: bool,
) -> Result<Vec<ResolvedPackage>, YarnValidationError> {
    let root = match serde_yaml::from_str::<serde_yaml::Value>(content) {
        Ok(root) => root,
        Err(_) if strict => return Err(YarnValidationError::Malformed),
        Err(_) => return Ok(Vec::new()),
    };
    let Some(map) = root.as_mapping() else {
        return (!strict)
            .then(Vec::new)
            .ok_or(YarnValidationError::Malformed);
    };
    let mut saw_metadata = false;
    let mut result = Vec::new();
    for (key, value) in map {
        let Some(key_str) = key.as_str().filter(|s| !s.is_empty()) else {
            if strict {
                return Err(YarnValidationError::Malformed);
            }
            continue;
        };
        if key_str == "__metadata" {
            if strict && !value.is_mapping() {
                return Err(YarnValidationError::Malformed);
            }
            saw_metadata = true;
            continue;
        }
        if strict && !value.is_mapping() {
            return Err(YarnValidationError::Malformed);
        }
        let pkg = parse_berry_entry(key_str, value);
        if strict && pkg.version.is_empty() && pkg.fingerprint.is_empty() {
            return Err(YarnValidationError::Malformed);
        }
        result.push(pkg);
    }
    if strict && !saw_metadata {
        return Err(YarnValidationError::Malformed);
    }
    Ok(result)
}

fn parse_berry_entry(key_str: &str, value: &serde_yaml::Value) -> ResolvedPackage {
    let name = extract_yarn_name(key_str);
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
    ResolvedPackage {
        name,
        version,
        fingerprint,
        kind: ResolutionKind::Registry,
    }
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

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
