use super::super::{ResolutionKind, ResolvedPackage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ComposerError;

pub(crate) fn parse(content: &str) -> Vec<ResolvedPackage> {
    parse_checked(content).unwrap_or_default()
}

pub(crate) fn parse_checked(content: &str) -> Result<Vec<ResolvedPackage>, ComposerError> {
    let root: serde_json::Value = serde_json::from_str(content).map_err(|_| ComposerError)?;
    let mut saw_section = false;
    let mut out = Vec::new();
    for key in ["packages", "packages-dev"] {
        if let Some(value) = root.get(key).filter(|value| !value.is_null()) {
            let packages = value.as_array().ok_or(ComposerError)?;
            saw_section = true;
            for entry in packages {
                out.push(parse_entry(entry)?);
            }
        }
    }
    if !saw_section {
        return Err(ComposerError);
    }
    Ok(out)
}

fn parse_entry(entry: &serde_json::Value) -> Result<ResolvedPackage, ComposerError> {
    let name = entry
        .get("name")
        .and_then(|value| value.as_str())
        .ok_or(ComposerError)?;
    let version = entry
        .get("version")
        .and_then(|value| value.as_str())
        .ok_or(ComposerError)?;
    let dist = package_source(entry, "dist")?;
    let source = package_source(entry, "source")?;
    let dist_type = text(dist, "type");
    let source_type = text(source, "type");
    let dist_ref = text(dist, "reference");
    let source_ref = text(source, "reference");
    let (kind, fingerprint) = if dist_type == "path" || source_type == "path" {
        let path = first_text(text(dist, "url"), text(source, "url"));
        let reference = first_text(dist_ref, source_ref);
        let fingerprint = match (path, reference) {
            ("", reference) => reference.to_string(),
            (path, "") => path.to_string(),
            (path, reference) => format!("{path}#{reference}"),
        };
        (ResolutionKind::Directory, fingerprint)
    } else if dist.is_none() && source_type == "git" {
        (ResolutionKind::Git, source_ref.to_string())
    } else {
        let fallback = first_text(source_ref, text(dist, "url"));
        let fingerprint = first_text(dist_ref, first_text(text(dist, "shasum"), fallback));
        (ResolutionKind::Registry, fingerprint.to_string())
    };
    Ok(ResolvedPackage {
        name: name.to_string(),
        version: version.to_string(),
        fingerprint,
        kind,
    })
}

fn package_source<'a>(
    entry: &'a serde_json::Value,
    key: &str,
) -> Result<Option<&'a serde_json::Value>, ComposerError> {
    let Some(value) = entry.get(key).filter(|value| !value.is_null()) else {
        return Ok(None);
    };
    if !value.is_object()
        || ["type", "url", "reference", "shasum"]
            .into_iter()
            .filter_map(|field| value.get(field))
            .any(|field| !field.is_null() && !field.is_string())
        || !required_text(value, "type")
        || !required_text(value, "url")
    {
        return Err(ComposerError);
    }
    Ok(Some(value))
}

fn required_text(value: &serde_json::Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(|field| field.as_str())
        .is_some_and(|text| !text.is_empty())
}

fn text<'a>(value: Option<&'a serde_json::Value>, key: &str) -> &'a str {
    value
        .and_then(|entry| entry.get(key))
        .and_then(|entry| entry.as_str())
        .unwrap_or("")
}

fn first_text<'a>(preferred: &'a str, fallback: &'a str) -> &'a str {
    if preferred.is_empty() {
        fallback
    } else {
        preferred
    }
}
