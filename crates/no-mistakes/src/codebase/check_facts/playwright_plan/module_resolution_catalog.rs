use super::path_match::is_external_terminal;
use crate::codebase::analysis_session::PathInterner;
use dashmap::DashMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// One request-scoped catalog facade for Playwright module identity.
///
/// Its config and frozen visibility universe are both owned. That lets a
/// scope resolver persist safely without borrowing from its containing
/// `Arc<TsConfigCatalog>` (and therefore without self-referential unsafe
/// state). Each importer selects a catalog scope once, while classifications
/// keep workspace recognition as well as resolved targets.
pub(super) struct CatalogModuleResolver {
    catalog: Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
    pub(super) universe: Arc<crate::codebase::ts_source::FrozenPathRemapper>,
    interner: Arc<PathInterner>,
    importer_scopes: DashMap<Arc<Path>, Option<usize>>,
    scopes: DashMap<Option<usize>, Arc<CatalogScopeResolver>>,
    pub(super) classifications: DashMap<(Arc<Path>, Arc<str>), CatalogClassification>,
    pub(super) scope_selections: AtomicUsize,
    pub(super) scope_builds: AtomicUsize,
}

struct CatalogScopeResolver {
    resolver: crate::codebase::ts_resolver::ImportResolver<'static>,
}

#[derive(Clone)]
pub(super) struct CatalogClassification {
    pub(super) import_classification: crate::codebase::ts_resolver::ImportClassification,
    pub(super) is_external_terminal: bool,
}

impl CatalogModuleResolver {
    pub(super) fn new(
        catalog: Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
        universe: Arc<crate::codebase::ts_source::FrozenPathRemapper>,
    ) -> Self {
        Self {
            catalog,
            universe,
            interner: Arc::new(PathInterner::new()),
            importer_scopes: DashMap::new(),
            scopes: DashMap::new(),
            classifications: DashMap::new(),
            scope_selections: AtomicUsize::new(0),
            scope_builds: AtomicUsize::new(0),
        }
    }

    pub(super) fn classify(
        &self,
        specifier: &str,
        importer: &Path,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    ) -> CatalogClassification {
        let importer = self.interner.intern_path(importer);
        let specifier_key = self.interner.intern_str(specifier);
        let key = (Arc::clone(&importer), Arc::clone(&specifier_key));
        self.classifications
            .entry(key)
            .or_insert_with(|| {
                let scope = self.scope_for(&importer);
                let import_classification = scope.resolver.classify_import(
                    specifier,
                    &importer,
                    workspace,
                    &self.universe.shared_normalized_visible().as_ref(),
                );
                CatalogClassification {
                    is_external_terminal: is_external_terminal(&scope.resolver, specifier),
                    import_classification,
                }
            })
            .clone()
    }

    fn scope_for(&self, importer: &Arc<Path>) -> Arc<CatalogScopeResolver> {
        let index = *self
            .importer_scopes
            .entry(Arc::clone(importer))
            .or_insert_with(|| {
                self.scope_selections.fetch_add(1, Ordering::Relaxed);
                self.catalog.resolver_scope_index_for(importer)
            });
        self.scopes
            .entry(index)
            .or_insert_with(|| {
                self.scope_builds.fetch_add(1, Ordering::Relaxed);
                let (config, _, _) = self.catalog.resolver_scope_at(index);
                Arc::new(CatalogScopeResolver {
                    resolver: crate::codebase::ts_resolver::ImportResolver::new_owned(Arc::new(
                        config.clone(),
                    ))
                    .with_owned_visible(self.universe.shared_normalized_visible())
                    .with_interner(Arc::clone(&self.interner)),
                })
            })
            .clone()
    }
}
