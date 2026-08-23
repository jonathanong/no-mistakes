use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod metadata;
use metadata::action_directory_valid;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum LocalActionKind {
    Docker,
    Other,
}

#[derive(Default)]
pub(crate) struct LocalActionCatalog(BTreeMap<String, LocalActionKind>);

impl LocalActionCatalog {
    pub(super) fn kind(&self, directory: &str) -> Option<LocalActionKind> {
        self.0.get(directory).copied()
    }
}

pub(crate) fn catalog(
    root: &Path,
    tracked_paths: &[PathBuf],
    sources: &SourceStore,
) -> LocalActionCatalog {
    let tracked = tracked_paths
        .iter()
        .map(|path| relative_slash_path(root, path))
        .collect::<BTreeSet<_>>();
    let mut descriptor_paths = BTreeMap::<String, (bool, &PathBuf)>::new();
    for path in tracked_paths {
        let name = path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or_default();
        if !matches!(name, "action.yml" | "action.yaml") {
            continue;
        }
        let directory = relative_slash_path(
            root,
            path.parent()
                .expect("action metadata has a parent directory"),
        );
        let preferred = name == "action.yml";
        descriptor_paths
            .entry(directory)
            .and_modify(|(current_preferred, current)| {
                if preferred && !*current_preferred {
                    *current_preferred = true;
                    *current = path;
                }
            })
            .or_insert((preferred, path));
    }
    let descriptors = descriptor_paths
        .into_iter()
        .filter_map(|(directory, (_, path))| {
            let source = sources.read_path(path).ok()?;
            let metadata = serde_yaml::from_str(&source).ok()?;
            Some((directory, metadata))
        })
        .collect::<BTreeMap<_, _>>();
    let mut cache = BTreeMap::new();
    LocalActionCatalog(
        descriptors
            .iter()
            .filter(|(directory, _)| {
                action_directory_valid(
                    directory,
                    &descriptors,
                    &tracked,
                    &mut BTreeSet::new(),
                    &mut cache,
                )
            })
            .map(|(directory, metadata)| {
                let kind = if metadata
                    .get("runs")
                    .and_then(|runs| runs.get("using"))
                    .and_then(Value::as_str)
                    == Some("docker")
                {
                    LocalActionKind::Docker
                } else {
                    LocalActionKind::Other
                };
                (directory.clone(), kind)
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests;
