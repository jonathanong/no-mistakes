use super::super::{ResolutionKind, ResolvedPackage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PipfileError;

pub(crate) fn parse(content: &str) -> Vec<ResolvedPackage> {
    parse_checked(content).unwrap_or_default()
}

pub(crate) fn parse_checked(content: &str) -> Result<Vec<ResolvedPackage>, PipfileError> {
    let root: serde_json::Value = serde_json::from_str(content).map_err(|_| PipfileError)?;
    let categories = root.as_object().ok_or(PipfileError)?;
    let mut saw_section = false;
    let mut out = Vec::new();
    for (name, value) in categories {
        if name == "_meta" {
            continue;
        }
        let section = value.as_object().ok_or(PipfileError)?;
        saw_section = true;
        for (name, entry) in section {
            out.push(parse_entry(name, entry)?);
        }
    }
    if !saw_section {
        return Err(PipfileError);
    }
    Ok(out)
}

fn parse_entry(name: &str, entry: &serde_json::Value) -> Result<ResolvedPackage, PipfileError> {
    let obj = entry.as_object().ok_or(PipfileError)?;
    let version = text(obj, "version").map(|raw| raw.strip_prefix("==").unwrap_or(raw).to_string());
    let (kind, fingerprint, version_optional) = if let Some(git) = text(obj, "git") {
        let fingerprint = text(obj, "ref").unwrap_or(git).to_string();
        (ResolutionKind::Git, fingerprint, true)
    } else if let Some(path) = text(obj, "path") {
        (ResolutionKind::Directory, path.to_string(), true)
    } else if let Some(url) = ["hg", "svn", "bzr"]
        .into_iter()
        .find_map(|key| text(obj, key))
    {
        let fingerprint = text(obj, "ref")
            .map_or_else(|| url.to_string(), |reference| format!("{url}#{reference}"));
        (ResolutionKind::Other, fingerprint, true)
    } else if let Some(url) = text(obj, "file").or_else(|| text(obj, "url")) {
        (ResolutionKind::Tarball, url.to_string(), true)
    } else if obj.contains_key("editable") {
        (ResolutionKind::Directory, String::new(), true)
    } else {
        let fingerprint = obj
            .get("hashes")
            .and_then(|value| value.as_array())
            .and_then(|hashes| hashes.first())
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        (ResolutionKind::Registry, fingerprint, false)
    };
    let version = version.ok_or(PipfileError).or_else(|err| {
        if version_optional {
            Ok(String::new())
        } else {
            Err(err)
        }
    })?;
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
