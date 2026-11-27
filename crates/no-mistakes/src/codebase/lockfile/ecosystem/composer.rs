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
    let dist = entry.get("dist");
    let source = entry.get("source");
    let dist_type = text(dist, "type");
    let source_type = text(source, "type");
    let dist_ref = text(dist, "reference");
    let source_ref = text(source, "reference");
    let (kind, fingerprint) = if dist_type == "path" || source_type == "path" {
        let fingerprint = if dist_ref.is_empty() {
            source_ref
        } else {
            dist_ref
        };
        (ResolutionKind::Directory, fingerprint)
    } else if dist.is_none() && source_type == "git" {
        (ResolutionKind::Git, source_ref)
    } else {
        let fingerprint = if dist_ref.is_empty() {
            source_ref
        } else {
            dist_ref
        };
        (ResolutionKind::Registry, fingerprint)
    };
    Ok(ResolvedPackage {
        name: name.to_string(),
        version: version.to_string(),
        fingerprint: fingerprint.to_string(),
        kind,
    })
}

fn text<'a>(value: Option<&'a serde_json::Value>, key: &str) -> &'a str {
    value
        .and_then(|entry| entry.get(key))
        .and_then(|entry| entry.as_str())
        .unwrap_or("")
}
