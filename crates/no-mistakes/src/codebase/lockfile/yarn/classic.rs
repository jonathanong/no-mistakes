use super::{extract_yarn_name, ResolutionKind, ResolvedPackage, YarnValidationError};

pub(super) fn parse_classic_inner(
    content: &str,
    strict: bool,
) -> Result<Vec<ResolvedPackage>, YarnValidationError> {
    let mut result = Vec::new();
    let mut saw_yarn_header = false;
    let mut current_name: Option<String> = None;
    let mut current_version = String::new();
    let mut current_integrity = String::new();
    let mut current_resolved = String::new();

    for raw_line in content.lines() {
        let line = raw_line.trim_end_matches('\r');
        let trimmed_start = line.trim_start();
        if trimmed_start.is_empty() || trimmed_start.starts_with('#') {
            saw_yarn_header |= trimmed_start.starts_with("# yarn lockfile");
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
    if strict && result.is_empty() && !saw_yarn_header {
        return Err(YarnValidationError::Malformed);
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

pub(super) fn extract_classic_name(header: &str) -> String {
    let first = header.split_once(", ").map_or(header, |(h, _)| h);
    extract_yarn_name(first.trim_matches('"'))
}
