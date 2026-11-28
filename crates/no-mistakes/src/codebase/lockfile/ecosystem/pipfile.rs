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
    let editable = obj
        .get("editable")
        .map(|value| value.as_bool().ok_or(PipfileError))
        .transpose()?
        .unwrap_or_default();
    let hashes = locked_hashes(obj.get("hashes"))?;
    let version = obj
        .get("version")
        .map(|value| value.as_str().ok_or(PipfileError))
        .transpose()?
        .map(|raw| raw.strip_prefix("==").unwrap_or(raw).to_string());
    let git = text(obj, "git")?;
    let path = text(obj, "path")?;
    let reference = text(obj, "ref")?;
    let subdirectory = text(obj, "subdirectory")?;
    let file = text(obj, "file")?;
    let url = text(obj, "url")?;
    let hg = text(obj, "hg")?;
    let svn = text(obj, "svn")?;
    let bzr = text(obj, "bzr")?;
    let vcs = hg.or(svn).or(bzr);
    let (kind, fingerprint, version_optional) = if let Some(git) = git {
        let fingerprint =
            super::source_fingerprint(reference.unwrap_or(git), "subdirectory", subdirectory);
        (ResolutionKind::Git, fingerprint, true)
    } else if let Some(path) = path {
        (ResolutionKind::Directory, path.to_string(), true)
    } else if let Some(url) = vcs {
        let fingerprint =
            reference.map_or_else(|| url.to_string(), |reference| format!("{url}#{reference}"));
        let fingerprint = super::source_fingerprint(&fingerprint, "subdirectory", subdirectory);
        (ResolutionKind::Other, fingerprint, true)
    } else if let Some(url) = file.or(url) {
        let fingerprint = super::source_fingerprint(url, "subdirectory", subdirectory);
        (ResolutionKind::Tarball, fingerprint, true)
    } else if editable {
        (ResolutionKind::Directory, String::new(), true)
    } else {
        (
            ResolutionKind::Registry,
            super::hash_fingerprint(hashes.into_iter()),
            false,
        )
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

fn locked_hashes(value: Option<&serde_json::Value>) -> Result<Vec<&str>, PipfileError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    value
        .as_array()
        .ok_or(PipfileError)?
        .iter()
        .map(|hash| hash.as_str().ok_or(PipfileError))
        .collect()
}

fn text<'a>(
    obj: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Option<&'a str>, PipfileError> {
    match obj.get(key) {
        None => Ok(None),
        Some(value) => value.as_str().map(Some).ok_or(PipfileError),
    }
}
