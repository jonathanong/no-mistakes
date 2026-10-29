//! Files that some module consumes other than by named static imports.
use super::path_provenance;
use crate::codebase::dependencies::graph::TsFactLookup;
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
    /// A specifier that might name a file but resolves to none.
    unknown: bool,
}

impl Consumed {
    pub(super) fn new(
        facts: &dyn TsFactLookup,
        graph_files: &[PathBuf],
        resolution: &Resolution<'_>,
    ) -> Self {
        let visible: PathSet = graph_files.iter().cloned().collect();
        let fallback_workspace = IndexedWorkspaceMap::default();
        let workspace = resolution.workspace.unwrap_or(&fallback_workspace);
        let empty = TsConfig::default();
        let mut consumed = Self::default();
        for file in graph_files {
            let Some(provenance) = facts.get_ts_facts(file).and_then(path_provenance) else {
                continue;
            };
            if provenance.unsafe_modules.is_empty() {
                continue;
            }
            let config = resolution.catalog.map_or(&empty, |c| c.config_for(file));
            let resolver = ImportResolver::new(config);
            for specifier in &provenance.unsafe_modules {
                let class = resolver.classify_import(specifier, file, workspace, &visible);
                match class.preferred_path() {
                    Some(target) => consumed.insert(target),
                    None if class.is_unresolved_external() && external(&resolver, specifier) => {}
                    None => consumed.unknown |= !specifier.starts_with(['.', '/']),
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
        self.unknown
    }

    pub(super) fn contains(&self, file: &Path) -> bool {
        self.files.contains(&normalize_path(file))
    }
}

/// A bare package name no alias or workspace claims.
fn external(resolver: &ImportResolver<'_>, specifier: &str) -> bool {
    !specifier.starts_with(['.', '/', '#']) && !resolver.matches_alias(specifier)
}
