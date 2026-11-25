use super::{ResolutionKind, ResolvedPackage};

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
        return if strict {
            Err(YarnValidationError::Malformed)
        } else {
            Ok(Vec::new())
        };
    };
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

fn parse_classic_inner(
    content: &str,
    strict: bool,
) -> Result<Vec<ResolvedPackage>, YarnValidationError> {
    let mut result = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_version = String::new();
    let mut current_integrity = String::new();
    let mut current_resolved = String::new();

    for raw_line in content.lines() {
        let line = raw_line.trim_end_matches('\r');
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            if let Some(name) = current_name.take() {
                result.push(finish_classic_entry(
                    name,
                    &mut current_version,
                    &mut current_integrity,
                    &mut current_resolved,
                    strict,
                )?);
            }
            let trimmed_end = line.trim_end();
            let header = match trimmed_end.strip_suffix(':') {
                Some(header) => header.trim_matches('"'),
                None if strict => return Err(YarnValidationError::Malformed),
                None => trimmed_end.trim_matches('"'),
            };
            let name = extract_classic_name(header);
            if strict && name.is_empty() {
                return Err(YarnValidationError::Malformed);
            }
            current_name = Some(name);
        } else {
            if strict && current_name.is_none() {
                return Err(YarnValidationError::Malformed);
            }
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
        result.push(finish_classic_entry(
            name,
            &mut current_version,
            &mut current_integrity,
            &mut current_resolved,
            strict,
        )?);
    }
    Ok(result)
}

fn finish_classic_entry(
    name: String,
    version: &mut String,
    integrity: &mut String,
    resolved: &mut String,
    strict: bool,
) -> Result<ResolvedPackage, YarnValidationError> {
    let fingerprint = if integrity.is_empty() {
        std::mem::take(resolved)
    } else {
        resolved.clear();
        std::mem::take(integrity)
    };
    let pkg = ResolvedPackage {
        name,
        version: std::mem::take(version),
        fingerprint,
        kind: ResolutionKind::Registry,
    };
    if strict && pkg.version.is_empty() && pkg.fingerprint.is_empty() {
        return Err(YarnValidationError::Malformed);
    }
    Ok(pkg)
}

fn extract_classic_name(header: &str) -> String {
    let first = header.split_once(", ").map_or(header, |(h, _)| h);
    extract_yarn_name(first.trim_matches('"'))
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
