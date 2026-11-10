//! Files that some module consumes other than by named static imports.
use super::path_provenance;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId, TsFactLookup};
use crate::codebase::ts_resolver::{
    normalize_path, ImportResolution, ImportResolver, ScopedImportResolver, TsConfig,
    TsConfigCatalog, VisiblePathLookup,
};
use crate::codebase::workspaces::IndexedWorkspaceMap;
use crate::fx::{FxHashSet, PathSet};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

#[path = "consumers_resolve.rs"]
mod resolve;
#[path = "consumers_scan.rs"]
mod scan;

pub(crate) struct Resolution<'a> {
    pub(crate) catalog: Option<&'a TsConfigCatalog>,
    pub(crate) workspace: Option<&'a IndexedWorkspaceMap>,
    pub(crate) session: Option<&'a AnalysisSession>,
}

impl<'a> Resolution<'a> {
    pub(crate) fn new(
        catalog: Option<&'a TsConfigCatalog>,
        prepared: Option<&'a crate::codebase::dependencies::graph::PreparedGraphConfig>,
    ) -> Self {
        Self {
            catalog,
            workspace: prepared.map(|prepared| prepared.workspace()),
            session: None,
        }
    }

    pub(crate) fn with_session(self, session: Option<&'a AnalysisSession>) -> Self {
        Self { session, ..self }
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
    scan_memo: RefCell<scan::ScanMemo>,
}

impl Consumed {
    pub(super) fn new(
        graph: &DepGraph,
        facts: &dyn TsFactLookup,
        graph_files: &[PathBuf],
        reachable_files: &FxHashSet<PathBuf>,
        resolution: &Resolution<'_>,
    ) -> Self {
        let path_set: PathSet = graph_files.iter().cloned().collect();
        let normalized = resolution
            .session
            .map(|session| session.normalized_visible(&path_set));
        let visible: &dyn VisiblePathLookup = match &normalized {
            Some(paths) => paths.as_ref(),
            None => &path_set,
        };
        let scoped = match (resolution.session, resolution.catalog) {
            (Some(session), Some(catalog)) => Some(ScopedImportResolver::new_in_session(
                catalog, visible, session,
            )),
            _ => None,
        };
        let fallback_workspace = IndexedWorkspaceMap::default();
        let workspace = resolution.workspace.unwrap_or(&fallback_workspace);
        let empty = TsConfig::default();
        let mut consumed = Self::default();
        let resolver_name = if scoped.is_some() {
            "session-scoped"
        } else {
            "private"
        };
        for file in graph_files
            .iter()
            .filter(|file| reachable_files.contains(*file))
        {
            let Some(ts) = facts.get_ts_facts(file) else {
                continue;
            };
            let Some(provenance) = path_provenance(ts) else {
                if ts.module_bindings.is_some() {
                    resolve::note_dropped(&mut consumed, file, ts);
                }
                continue;
            };
            let config = resolution.catalog.map_or(&empty, |c| c.config_for(file));
            let private = scoped.is_none().then(|| ImportResolver::new(config));
            let resolver: &dyn ImportResolution = match scoped.as_ref() {
                Some(scoped) => scoped,
                None => private.as_ref().expect("private resolver"),
            };
            let gate = resolve::Gate {
                resolver,
                workspace,
                visible,
                resolver_name,
            };
            resolve::apply(&gate, &mut consumed, file, ts, provenance);
            if let Some(module) = ts.module_bindings.as_deref() {
                super::route_index::note_nondirect(
                    &mut consumed.hits,
                    file,
                    provenance,
                    module,
                    resolver,
                    workspace,
                    visible,
                );
            }
        }
        link_barrels(&mut consumed, graph, facts);
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
}

fn link_barrels(consumed: &mut Consumed, graph: &DepGraph, facts: &dyn TsFactLookup) {
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
}

#[cfg(test)]
#[path = "consumers/tests.rs"]
mod tests;
