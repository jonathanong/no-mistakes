use super::super::{ResolutionKind, ResolvedPackage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PipfileError;

pub(crate) fn parse(content: &str) -> Vec<ResolvedPackage> {
    parse_checked(content).unwrap_or_default()
}

pub(crate) fn parse_checked(content: &str) -> Result<Vec<ResolvedPackage>, PipfileError> {
    let root: serde_json::Value = serde_json::from_str(content).map_err(|_| PipfileError)?;
    let mut saw_section = false;
    let mut out = Vec::new();
    for key in ["default", "develop"] {
        if let Some(value) = root.get(key) {
            let section = value.as_object().ok_or(PipfileError)?;
            saw_section = true;
            for (name, entry) in section {
                out.push(parse_entry(name, entry)?);
            }
        }
    }
    if !saw_section {
        return Err(PipfileError);
    }
    Ok(out)
}

fn parse_entry(name: &str, entry: &serde_json::Value) -> Result<ResolvedPackage, PipfileError> {
    let obj = entry.as_object().ok_or(PipfileError)?;
    let version_raw = obj
        .get("version")
        .and_then(|value| value.as_str())
        .ok_or(PipfileError)?;
    let version = version_raw
        .strip_prefix("==")
        .unwrap_or(version_raw)
        .to_string();
    let (kind, fingerprint) = if let Some(git) = text(obj, "git") {
        let fingerprint = text(obj, "ref").unwrap_or(git).to_string();
        (ResolutionKind::Git, fingerprint)
    } else if let Some(path) = text(obj, "path") {
        (ResolutionKind::Directory, path.to_string())
    } else {
        let fingerprint = obj
            .get("hashes")
            .and_then(|value| value.as_array())
            .and_then(|hashes| hashes.first())
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        (ResolutionKind::Registry, fingerprint)
    };
    Ok(ResolvedPackage {
        name: name.to_string(),
        version,
        fingerprint,
        kind,
    })
}

fn text<'a>(obj: &'a serde_json::Map<String, serde_json::Value>, key: &str) -> Option<&'a str> {
    obj.get(key).and_then(|value| value.as_str())
}
