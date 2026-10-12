use super::{manifest, workspace, PackageNode};
use crate::codebase::{ts_source::SourceStore, workspaces::WorkspaceMap};
use anyhow::Result;
use dashmap::{mapref::entry::Entry, DashMap};
use rustc_hash::FxBuildHasher;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

type Nodes = BTreeMap<String, PackageNode>;

// Root, skip policy, visible files, and sources are fixed by this invocation.
// Preserve target-root order because duplicate package names use that order.
#[derive(Default)]
pub(super) struct SelectionCache {
    entries: DashMap<Vec<PathBuf>, Arc<WorkspaceSelection>, FxBuildHasher>,
}

pub(super) struct WorkspaceSelection {
    pub(super) workspace: WorkspaceMap,
    manifests: DashMap<Vec<String>, Arc<Nodes>, FxBuildHasher>,
}

impl SelectionCache {
    pub(super) fn get_or_compute(
        &self,
        root: &Path,
        target_roots: &[PathBuf],
        files: &[PathBuf],
        sources: &SourceStore,
    ) -> Result<Arc<WorkspaceSelection>> {
        match self.entries.entry(target_roots.to_vec()) {
            Entry::Occupied(entry) => Ok(Arc::clone(entry.get())),
            Entry::Vacant(entry) => {
                // Return the original anyhow error, retaining its context chain.
                let workspace =
                    workspace::load_workspace_with_sources(root, target_roots, files, sources)?;
                let selection = Arc::new(WorkspaceSelection {
                    workspace,
                    manifests: DashMap::with_hasher(FxBuildHasher),
                });
                entry.insert(Arc::clone(&selection));
                Ok(selection)
            }
        }
    }
}

impl WorkspaceSelection {
    pub(super) fn manifest_nodes(
        &self,
        dependency_types: &[&str],
        sources: &SourceStore,
    ) -> Arc<Nodes> {
        let key = dependency_types
            .iter()
            .map(|field| field.to_string())
            .collect();
        Arc::clone(
            self.manifests
                .entry(key)
                .or_insert_with(|| {
                    Arc::new(manifest::manifest_nodes_with_sources(
                        &self.workspace,
                        dependency_types,
                        sources,
                    ))
                })
                .value(),
        )
    }
}
