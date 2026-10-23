use dashmap::DashMap as ScopedDashMap;
use dashmap::mapref::entry::Entry as ScopedEntry;
use std::path::{Path as ScopedPath, PathBuf as ScopedPathBuf};
use std::sync::Arc as ScopedArc;
use std::sync::atomic::{AtomicUsize as ScopedAtomicUsize, Ordering as ScopedOrdering};

#[path = "scoped_facade.rs"]
mod facade;
pub(crate) use facade::{ImportResolution, ImportResolverFacade, WorkspaceFallbackResolver};

/// Identity of one selected catalog resolver scope within this facade.
///
/// The immutable catalog provides stable addresses for selected configurations.
/// Session cache keys retain exact visible identities; standalone scopes retain
/// full path keys. Both are constructed once per selected configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ScopedResolverSelectionKey {
    config: usize,
    module_resolution: Option<(usize, usize)>,
    identity: (usize, usize),
}

impl ScopedResolverSelectionKey {
    fn new(config: &TsConfig, module_resolution: Option<&str>, identity: &[ScopedPathBuf]) -> Self {
        Self {
            config: config as *const TsConfig as usize,
            module_resolution: module_resolution
                .map(|value| (value.as_ptr() as usize, value.len())),
            identity: (identity.as_ptr() as usize, identity.len()),
        }
    }
}

struct ImporterSelectionCache {
    selections: ScopedDashMap<ScopedPathBuf, Option<usize>>,
    builds: ScopedAtomicUsize,
}

/// A resolver facade that chooses one cached `ImportResolver` scope for each
/// importer. It owns a normalized visibility view, so symlinked workspace
/// paths and real config identities share cached resolver outcomes.
pub(crate) struct ScopedImportResolver<'a> {
    catalog: &'a TsConfigCatalog,
    visible: Option<ResolverVisible<'a>>,
    fixed_resolver: Option<ImportResolver<'a>>,
    automatic_fixed_roots: Option<(ScopedPathBuf, ScopedPathBuf)>,
    caches: ScopedDashMap<ResolverCacheScopeKey, ScopedArc<ResolverResultCache>>,
    scope_caches: ScopedDashMap<ScopedResolverSelectionKey, ScopedArc<ResolverResultCache>>,
    importer_selections: ScopedArc<ImporterSelectionCache>,
    scope_key_builds: ScopedAtomicUsize,
    scope_cache_lookups: ScopedAtomicUsize,
    session: Option<&'a crate::codebase::analysis_session::AnalysisSession>,
    shared_cache: Option<&'a ImportResolutionCache>,
    queue_root: Option<&'a ScopedPath>,
}

impl<'a> ScopedImportResolver<'a> {
    pub(crate) fn with_shared_cache(mut self, cache: &'a ImportResolutionCache) -> Self {
        if let Some(resolver) = self.fixed_resolver.as_mut() {
            resolver.shared_cache = Some(cache);
        }
        self.shared_cache = Some(cache);
        self
    }

    /// None means no matching paths pattern; Some(None) means its target is unresolved.
    pub(crate) fn resolve_path_alias(
        &self,
        specifier: &str,
        importing_file: &ScopedPath,
    ) -> Option<Option<ScopedPathBuf>> {
        let resolve = |resolver: &ImportResolver<'_>| {
            if !resolver.matches_alias(specifier) {
                return None;
            }
            let Some(target) = resolver.resolve(specifier, importing_file) else {
                return Some(None);
            };
            // A matching paths pattern may fail and resolve through baseUrl.
            // Accept only mapped candidates, while sharing the ordinary cache.
            Some(
                resolver
                    .path_alias_candidates(specifier)
                    .contains(&target)
                    .then_some(target),
            )
        };
        if let Some(resolver) = self.fixed_resolver_for(importing_file) {
            return resolve(resolver);
        }
        resolve(&self.resolver_for(importing_file))
    }

    pub(crate) fn resolve(
        &self,
        specifier: &str,
        importing_file: &ScopedPath,
    ) -> Option<ScopedPathBuf> {
        if let Some(resolver) = self.fixed_resolver_for(importing_file) {
            return resolver.resolve(specifier, importing_file);
        }
        self.resolver_for(importing_file)
            .resolve(specifier, importing_file)
    }

    /// Resolve every candidate with the TypeScript configuration selected for
    /// this importer, including targets deleted before planning.
    pub(crate) fn resolution_candidates(
        &self,
        specifier: &str,
        importing_file: &ScopedPath,
    ) -> std::collections::BTreeSet<ScopedPathBuf> {
        if let Some(resolver) = self.fixed_resolver_for(importing_file) {
            return resolver.resolution_candidates(specifier, importing_file);
        }
        self.resolver_for(importing_file)
            .resolution_candidates(specifier, importing_file)
    }

    pub(crate) fn classify_import(
        &self,
        specifier: &str,
        importing_file: &ScopedPath,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &dyn VisiblePathLookup,
    ) -> ImportClassification {
        if let Some(resolver) = self.fixed_resolver_for(importing_file) {
            return resolver.classify_import(specifier, importing_file, workspace, visible_files);
        }
        self.resolver_for(importing_file).classify_import(
            specifier,
            importing_file,
            workspace,
            visible_files,
        )
    }

    fn resolver_for(&self, importing_file: &ScopedPath) -> ImportResolver<'_> {
        let selection = self.importer_selection(importing_file);
        let (config, module_resolution, identity) = self.catalog.resolver_scope_at(selection);
        let observer = match self.session {
            Some(session) => session.observer().cloned(),
            None => crate::diagnostics::current(),
        };
        let mut resolver = ImportResolver::new_observed(config, observer);
        if let Some(visible) = self.visible.as_ref() {
            resolver.visible = Some(visible.clone());
        }
        if let Some(session) = self.session {
            resolver.interner = Some(session.interner_arc());
        }
        if let Some(root) = self.queue_root {
            resolver = resolver.with_queue_compatibility(root);
        }
        if let Some(shared_cache) = self.shared_cache {
            if self.session.is_none() {
                return resolver.with_shared_cache(shared_cache);
            }
            resolver.shared_cache = Some(shared_cache);
        }
        let selection = ScopedResolverSelectionKey::new(config, module_resolution, identity);
        let cache = self
            .scope_caches
            .entry(selection)
            .or_insert_with(|| {
                self.scope_key_builds.fetch_add(1, ScopedOrdering::Relaxed);
                let key = ResolverCacheScopeKey::for_lookup(
                    config,
                    self.visible.as_ref().map(ResolverVisible::lookup),
                    module_resolution,
                    identity,
                )
                .with_queue_root(self.queue_root);
                // The exact, owned key reaches the registry only once per
                // selected scope. Repeated importers clone this Arc directly,
                // avoiding both a full visible-set clone and its hash.
                self.scope_cache_lookups
                    .fetch_add(1, ScopedOrdering::Relaxed);
                match self.session {
                    Some(session) => session.resolver_cache_for_scope(key),
                    None => match self.caches.entry(key) {
                        ScopedEntry::Occupied(entry) => ScopedArc::clone(entry.get()),
                        ScopedEntry::Vacant(entry) => {
                            let cache =
                                ScopedArc::new(DashMap::with_hasher(rustc_hash::FxBuildHasher));
                            entry.insert(ScopedArc::clone(&cache));
                            cache
                        }
                    },
                }
            })
            .clone();
        resolver.cache = cache;
        resolver.session_scoped = true;
        resolver.session_exact_visibility = self.session.is_some() && self.visible.is_some();
        resolver
    }
}
