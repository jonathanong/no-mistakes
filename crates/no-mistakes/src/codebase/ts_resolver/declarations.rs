//! Supplemental declaration projection of the request's selected resolver.
//! Ordinary import resolution keeps its original file universe and outputs.
use super::{ImportResolution, ResolverCacheScopeKey, TsConfig, TsConfigCatalog};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::ts_source::SourceStore;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
#[path = "declarations/ambient.rs"]
mod ambient;
#[path = "declarations/package.rs"]
mod package;
#[path = "declarations/type_entries.rs"]
mod type_entries;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionKey {
    root: PathBuf,
    visible: ResolverCacheScopeKey,
    catalog: Option<super::CatalogProjectionKey>,
    workspace: Option<crate::codebase::workspaces::WorkspaceProjectionKey>,
    workspace_visible: Option<ResolverCacheScopeKey>,
}
pub(crate) type DeclarationResults = DashMap<(PathBuf, String), Arc<OnceLock<Option<PathBuf>>>>;

pub(crate) struct DeclarationResolver<'a> {
    session: &'a AnalysisSession,
    root: &'a Path,
    resolver: &'a dyn ImportResolution,
    sources: &'a SourceStore,
    results: Arc<DeclarationResults>,
    ambient: ambient::AmbientModules,
    type_ambient: OnceLock<ambient::AmbientModules>,
    facts: Option<&'a dyn crate::codebase::dependencies::graph::TsFactLookup>,
    catalog: Option<&'a TsConfigCatalog>,
    workspace: Option<&'a crate::codebase::workspaces::IndexedWorkspaceMap>,
}

impl<'a> DeclarationResolver<'a> {
    pub fn new(
        session: &'a AnalysisSession,
        root: &'a Path,
        resolver: &'a dyn ImportResolution,
        tsconfig: &TsConfig,
        catalog: Option<&'a TsConfigCatalog>,
        sources: &'a SourceStore,
        workspace: Option<&'a crate::codebase::workspaces::IndexedWorkspaceMap>,
    ) -> Self {
        // Select the complete catalog/visibility projection once, before any
        // importer or class traversal. Individual lookups reuse this result map.
        let key = ProjectionKey {
            root: root.to_path_buf(),
            visible: ResolverCacheScopeKey::for_lookup(
                tsconfig,
                resolver.visible_files(),
                None,
                &[],
            ),
            catalog: catalog.map(TsConfigCatalog::projection_key),
            workspace: workspace.map(|workspace| workspace.projection_key()),
            workspace_visible: resolver.workspace_visible_files().map(|visible| {
                ResolverCacheScopeKey::for_lookup(tsconfig, Some(visible), None, &[])
            }),
        };
        let results = session
            .declaration_resolutions
            .entry(key)
            .or_default()
            .clone();
        Self {
            session,
            root,
            resolver,
            sources,
            results,
            ambient: Default::default(),
            type_ambient: OnceLock::new(),
            facts: None,
            catalog,
            workspace,
        }
    }

    pub fn with_ambient_modules(
        mut self,
        paths: &[PathBuf],
        facts: &'a dyn crate::codebase::dependencies::graph::TsFactLookup,
        catalog: Option<&TsConfigCatalog>,
    ) -> Self {
        self.facts = Some(facts);
        self.ambient = ambient::AmbientModules::new(paths, facts, catalog);
        self
    }

    pub fn ambient_class(
        &self,
        specifier: &str,
        importer: &Path,
        exported: &str,
    ) -> Option<(PathBuf, crate::codebase::dependencies::extract::CallableId)> {
        match self
            .ambient
            .class(specifier, importer, exported, self.catalog)
        {
            ambient::ClassLookup::Unique(path, id) => return Some((path, id)),
            // An ambiguous in-graph module is not proved by a unique type entry.
            ambient::ClassLookup::Ambiguous => return None,
            ambient::ClassLookup::Missing => {}
        }
        match self
            .type_modules()
            .class(specifier, importer, exported, self.catalog)
        {
            ambient::ClassLookup::Unique(path, id) => Some((path, id)),
            ambient::ClassLookup::Ambiguous | ambient::ClassLookup::Missing => None,
        }
    }

    fn type_modules(&self) -> &ambient::AmbientModules {
        if self.type_ambient.get().is_some() {
            return self
                .type_ambient
                .get()
                .expect("type-visible ambient modules were just initialized");
        }
        let modules = type_entries::visible_modules(
            self.session,
            self.root,
            self.sources,
            self.catalog,
            self.facts,
        );
        self.type_ambient.get_or_init(|| modules)
    }

    pub fn resolve(&self, specifier: &str, importer: &Path) -> Option<PathBuf> {
        let cell = self
            .results
            .entry((importer.to_path_buf(), specifier.into()))
            .or_default()
            .clone();
        cell.get_or_init(|| {
            self.session
                .record_work("resolver.declaration_resolutions", 1);
            self.resolve_uncached(specifier, importer)
        })
        .clone()
    }

    fn resolve_uncached(&self, specifier: &str, importer: &Path) -> Option<PathBuf> {
        if let Some(path) = self.resolver.resolve(specifier, importer) {
            let declaration = [".d.ts", ".d.mts", ".d.cts"]
                .iter()
                .any(|suffix| path.to_string_lossy().ends_with(suffix));
            if !declaration {
                return Some(path);
            }
            let context = package::lookup_context(specifier, importer, self.root)?;
            if context == importer {
                return Some(path);
            }
            return (specifier.starts_with("./") || specifier.starts_with("../"))
                .then(|| package::resolve_in_context(specifier, importer, self.root, self.sources))
                .flatten();
        }
        if self
            .workspace
            .is_some_and(|workspace| workspace.recognizes_specifier_from(specifier, importer))
        {
            return None;
        }
        // A real local candidate outside the selected graph still shadows an
        // external namesake; declaration fallback must not change ownership.
        if !specifier.starts_with('.')
            && self
                .resolver
                .resolution_candidates(specifier, importer)
                .iter()
                .any(|path| path.is_file())
        {
            return None;
        }
        package::resolve_in_context(specifier, importer, self.root, self.sources)
    }
}
