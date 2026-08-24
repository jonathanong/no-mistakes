use super::DEFAULT_DEPENDENCY_FIELDS;
use crate::codebase::package_deps;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(super) struct Manifest {
    pub(super) path: PathBuf,
    pub(super) dir: PathBuf,
    pub(super) name: Option<String>,
}

pub(super) fn collect(root: &Path, files: &[PathBuf], sources: &SourceStore) -> Vec<Manifest> {
    let mut manifests = files
        .iter()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("package.json"))
        .filter_map(|path| {
            let value = sources.parse_json_path(path).ok()?;
            let name = value
                .get("name")
                .and_then(|value| value.as_str())
                .map(str::to_string);
            Some(Manifest {
                path: path.clone(),
                dir: path.parent()?.to_path_buf(),
                name,
            })
        })
        .collect::<Vec<_>>();
    manifests.sort_by_key(|manifest| relative_slash_path(root, &manifest.path));
    manifests
}

pub(super) fn dependency_fields(configured: &[String]) -> Vec<&str> {
    if configured.is_empty() {
        DEFAULT_DEPENDENCY_FIELDS.to_vec()
    } else {
        configured.iter().map(String::as_str).collect()
    }
}

pub(super) fn matching_dependencies(
    manifest: &Manifest,
    prefixes: &[String],
    fields: &[&str],
    sources: &SourceStore,
) -> BTreeSet<String> {
    package_deps::dependency_entries_from_source_store(&manifest.path, fields, sources)
        .into_iter()
        .filter(|dep| prefixes.iter().any(|prefix| dep.name.starts_with(prefix)))
        .map(|dep| dep.name)
        .collect()
}
