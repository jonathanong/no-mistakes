use crate::codebase::ts_source::{JsonLoadError, SourceStore};
use std::io::ErrorKind;
use std::path::Path;

type Version = (u64, u64);

pub(super) fn compiler_version(consumer_root: &Path, sources: &SourceStore) -> Option<Version> {
    // Stop at the first present/failed manifest; invalid nearer metadata must
    // never select an unrelated ancestor compiler. Missing manifests may ascend.
    consumer_root.ancestors()
        .map(|directory| sources.parse_json_path(&directory.join("node_modules/typescript/package.json")))
        .find(|result| !matches!(result, Err(JsonLoadError::Io(error)) if error.kind() == ErrorKind::NotFound))
        .and_then(Result::ok)
        .and_then(|manifest| {
            let text = manifest.get("version")?.as_str()?;
            let parts: Vec<_> = text.split('.').collect();
            let [major, minor, patch] = parts.as_slice() else { return None; };
            let version = (major.parse().ok()?, minor.parse().ok()?);
            patch.parse::<u64>().ok()?;
            Some(version)
        })
}

pub(super) fn applicable(range: &str, version: impl FnOnce() -> Option<Version>) -> Option<bool> {
    let (operator, bound) = ["<=", ">=", "<", ">", "="]
        .into_iter()
        .find_map(|operator| range.strip_prefix(operator).map(|bound| (operator, bound)))?;
    let (major, minor) = bound.split_once('.')?;
    let bound: Version = (major.parse().ok()?, minor.parse().ok()?);
    let version = version()?;
    Some(match operator {
        "<=" => version <= bound,
        ">=" => version >= bound,
        "<" => version < bound,
        ">" => version > bound,
        _ => version == bound,
    })
}
