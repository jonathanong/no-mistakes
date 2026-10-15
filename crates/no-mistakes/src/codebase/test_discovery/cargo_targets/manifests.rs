use crate::codebase::ts_source::SourceStore;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Parsed successes and failures have one request-local identity, including
/// workspace manifests needed by multiple packages with inherited editions.
pub(super) struct Manifests<'a> {
    sources: &'a SourceStore,
    values: BTreeMap<PathBuf, Option<toml::Value>>,
}
impl<'a> Manifests<'a> {
    pub(super) fn new(sources: &'a SourceStore) -> Self {
        Self {
            sources,
            values: BTreeMap::new(),
        }
    }
    pub(super) fn read(&mut self, path: &Path) -> Option<&toml::Value> {
        self.values
            .entry(path.to_path_buf())
            .or_insert_with(|| {
                self.sources
                    .read_path(path)
                    .ok()
                    .and_then(|source| toml::from_str(&source).ok())
            })
            .as_ref()
    }
    pub(super) fn edition(&mut self, path: &Path, visible: &BTreeSet<&Path>) -> Option<String> {
        let package = self.read(path)?.get("package")?;
        let edition = package.get("edition")?;
        if let Some(edition) = edition.as_str() {
            return Some(edition.into());
        }
        if edition.get("workspace").and_then(toml::Value::as_bool) != Some(true) {
            return None;
        }
        let workspace = package
            .get("workspace")
            .and_then(toml::Value::as_str)
            .map(str::to_owned);
        let directory = path.parent().expect("manifest filenames have a parent");
        let candidates: Vec<_> = workspace.map_or_else(
            || {
                directory
                    .ancestors()
                    .map(|parent| parent.join("Cargo.toml"))
                    .collect()
            },
            |workspace| {
                vec![crate::codebase::ts_resolver::normalize_path(
                    &directory.join(workspace).join("Cargo.toml"),
                )]
            },
        );
        for candidate in candidates {
            if !visible.contains(candidate.as_path()) {
                continue;
            }
            let Some(workspace) = self
                .read(&candidate)
                .and_then(|value| value.get("workspace"))
            else {
                continue;
            };
            return workspace
                .get("package")?
                .get("edition")?
                .as_str()
                .map(str::to_owned);
        }
        None
    }
}
