//! Files that some module consumes other than by named static imports.
use super::path_provenance;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId, TsFactLookup};
use crate::codebase::ts_resolver::{normalize_path, ImportResolver, TsConfig, TsConfigCatalog};
use crate::codebase::workspaces::IndexedWorkspaceMap;
use crate::fx::{FxHashSet, PathSet};
use std::path::{Path, PathBuf};

pub(crate) struct Resolution<'a> {
    pub(crate) catalog: Option<&'a TsConfigCatalog>,
    pub(crate) workspace: Option<&'a IndexedWorkspaceMap>,
}

impl<'a> Resolution<'a> {
    pub(crate) fn new(
        catalog: Option<&'a TsConfigCatalog>,
        prepared: Option<&'a crate::codebase::dependencies::graph::PreparedGraphConfig>,
    ) -> Self {
        Self {
            catalog,
            workspace: prepared.map(|prepared| prepared.workspace()),
        }
    }
}

#[derive(Default)]
pub(super) struct Consumed {
    files: FxHashSet<PathBuf>,
    /// Files whose specifier might name a module but resolves to none.
    unknown_files: FxHashSet<PathBuf>,
    hits: Vec<super::route_index::RouteHit>,
    unknown_sites: Vec<(PathBuf, u32)>,
    via: Vec<(PathBuf, PathBuf)>,
}

impl Consumed {
    pub(super) fn new(
        graph: &DepGraph,
        facts: &dyn TsFactLookup,
        graph_files: &[PathBuf],
        reachable_files: &FxHashSet<PathBuf>,
        resolution: &Resolution<'_>,
    ) -> Self {
        let visible: PathSet = graph_files.iter().cloned().collect();
        let fallback_workspace = IndexedWorkspaceMap::default();
        let workspace = resolution.workspace.unwrap_or(&fallback_workspace);
        let empty = TsConfig::default();
        let mut consumed = Self::default();
        for file in graph_files
            .iter()
            .filter(|file| reachable_files.contains(*file))
        {
            let Some(provenance) = facts.get_ts_facts(file).and_then(path_provenance) else {
                continue;
            };
            let config = resolution.catalog.map_or(&empty, |c| c.config_for(file));
            let resolver = ImportResolver::new(config);
            for specifier in &provenance.unsafe_modules {
                let class = resolver.classify_import(specifier, file, workspace, &visible);
                match class.preferred_path() {
                    Some(target) => {
                        super::route_index::note_resolved(
                            &mut consumed.hits,
                            file,
                            provenance,
                            specifier,
                            target,
                        );
                        consumed.insert(target);
                    }
                    None if class.is_unresolved_external() && external(&resolver, specifier) => {}
                    None => {
                        let line = super::route_index::unknown_line(provenance, specifier);
                        consumed.unknown_files.insert(file.clone());
                        consumed.unknown_sites.push((file.clone(), line));
                    }
                }
            }
            if let Some(module) = facts
                .get_ts_facts(file)
                .and_then(|facts| facts.module_bindings.as_deref())
            {
                super::route_index::note_nondirect(
                    &mut consumed.hits,
                    file,
                    provenance,
                    module,
                    &resolver,
                    workspace,
                    &visible,
                );
            }
        }
        let mut pending: Vec<_> = consumed.files.iter().cloned().collect();
        while let Some(file) = pending.pop() {
            let barrel = facts
                .get_ts_facts(&file)
                .and_then(|facts| facts.module_bindings.as_deref())
                .is_some_and(|module| {
                    module
                        .exports
                        .iter()
                        .any(|export| !export.type_only && export.specifier.is_some())
                });
            if !barrel {
                continue;
            }
            for (target, kind) in graph
                .dependencies_of_node(&NodeId::file(&file))
                .into_iter()
                .flatten()
            {
                if !matches!(kind, EdgeKind::Import | EdgeKind::WorkspaceImport) {
                    continue;
                }
                if let NodeId::File(target) = target {
                    if !consumed.contains(target) {
                        super::route_index::link(&mut consumed.via, target, &file);
                        consumed.insert(target);
                        pending.push(target.to_path_buf());
                    }
                }
            }
        }
        consumed
    }

    fn insert(&mut self, target: &Path) {
        self.files.insert(normalize_path(target));
        if let Ok(real) = target.canonicalize() {
            self.files.insert(normalize_path(&real));
        }
    }

    pub(super) fn unknown(&self) -> bool {
        !self.unknown_files.is_empty()
    }

    pub(super) fn contains(&self, file: &Path) -> bool {
        self.files.contains(&normalize_path(file))
    }

    pub(super) fn unknown_sites(&self) -> &[(PathBuf, u32)] {
        &self.unknown_sites
    }

    pub(super) fn targeting(&self, file: &Path) -> Vec<&super::route_index::RouteHit> {
        super::route_index::targeting(&self.hits, &self.via, file)
    }
}

/// A bare package name no alias or workspace claims.
fn external(resolver: &ImportResolver<'_>, specifier: &str) -> bool {
    !specifier.starts_with(['.', '/', '#']) && !resolver.matches_alias(specifier)
}

#[cfg(test)]
#[path = "consumers/tests.rs"]
mod tests;
