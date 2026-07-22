use dashmap::mapref::entry::Entry as ScopedEntry;
use dashmap::DashMap as ScopedDashMap;
use std::collections::HashSet as ScopedHashSet;
use std::path::{Path as ScopedPath, PathBuf as ScopedPathBuf};
use std::sync::atomic::{AtomicUsize as ScopedAtomicUsize, Ordering as ScopedOrdering};
use std::sync::Arc as ScopedArc;

#[path = "scoped_facade.rs"]
mod facade;
pub(crate) use facade::{ImportResolution, ImportResolverFacade};

/// Identity of one selected catalog resolver scope within this facade.
///
/// `ResolverCacheScopeKey` deliberately owns a sorted copy of the complete
/// visible universe so it can safely cross session boundaries. Rebuilding that
/// value for every importer made standalone scoped resolution quadratic in the
/// number of importers and visible files. The catalog is immutable for this
/// resolver's lifetime, so addresses into it are a stable, request-local way
/// to memoize the already-complete cache key per selected configuration.
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

/// A resolver facade that chooses one cached `ImportResolver` scope for each
/// importer. It owns a normalized visibility view, so symlinked workspace
/// paths and real config identities share cached resolver outcomes.
pub(crate) struct ScopedImportResolver<'a> {
    catalog: &'a TsConfigCatalog,
    visible: Option<ScopedArc<ScopedHashSet<ScopedPathBuf>>>,
    fixed_resolver: Option<ImportResolver<'a>>,
    automatic_fixed_roots: Option<(ScopedPathBuf, ScopedPathBuf)>,
    caches: ScopedDashMap<ResolverCacheScopeKey, ScopedArc<ResolverResultCache>>,
    scope_caches: ScopedDashMap<ScopedResolverSelectionKey, ScopedArc<ResolverResultCache>>,
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

    pub(crate) fn with_queue_compatibility(mut self, root: &'a ScopedPath) -> Self {
        self.fixed_resolver = self.fixed_resolver.take().map(|mut resolver| {
            // A fixed automatic resolver may currently point at the
            // session's standard-policy cache. Queue compatibility has
            // different relative/bare semantics, so detach before its
            // policy setter clears the cache.
            resolver.cache = ScopedArc::new(DashMap::new());
            resolver.shared_cache = None;
            resolver.with_queue_compatibility(root)
        });
        self.automatic_fixed_roots = None;
        self.shared_cache = None;
        self.queue_root = Some(root);
        self
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

    pub(crate) fn classify_import(
        &self,
        specifier: &str,
        importing_file: &ScopedPath,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
        visible_files: &ScopedHashSet<ScopedPathBuf>,
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
        let (config, module_resolution, identity) = self.catalog.resolver_scope_for(importing_file);
        let observer = match self.session {
            Some(session) => session.observer().cloned(),
            None => None,
        };
        let mut resolver = ImportResolver::new_observed(config, observer);
        if let Some(visible) = self.visible.as_ref() {
            resolver.visible = Some(ResolverVisible::Owned(ScopedArc::clone(visible)));
        }
        if let Some(shared_cache) = self.shared_cache {
            return resolver.with_shared_cache(shared_cache);
        }
        if let Some(root) = self.queue_root {
            resolver = resolver.with_queue_compatibility(root);
        }
        let selection = ScopedResolverSelectionKey::new(config, module_resolution, identity);
        let cache = self
            .scope_caches
            .entry(selection)
            .or_insert_with(|| {
                self.scope_key_builds.fetch_add(1, ScopedOrdering::Relaxed);
                let key = ResolverCacheScopeKey::new(
                    config,
                    self.visible.as_deref(),
                    module_resolution,
                    identity,
                );
                // The exact, owned key reaches the registry only once per
                // selected scope. Repeated importers clone this Arc directly,
                // avoiding both a full visible-set clone and its hash.
                self.scope_cache_lookups
                    .fetch_add(1, ScopedOrdering::Relaxed);
                match (self.session, self.queue_root.is_some()) {
                    (Some(session), false) => session.resolver_cache_for_scope(key),
                    _ => match self.caches.entry(key) {
                        ScopedEntry::Occupied(entry) => ScopedArc::clone(entry.get()),
                        ScopedEntry::Vacant(entry) => {
                            let cache = ScopedArc::new(DashMap::new());
                            entry.insert(ScopedArc::clone(&cache));
                            cache
                        }
                    },
                }
            })
            .clone();
        resolver.cache = cache;
        resolver.session_scoped = true;
        resolver
    }

    fn fixed_resolver_for(&self, importing_file: &ScopedPath) -> Option<&ImportResolver<'a>> {
        let resolver = self.fixed_resolver.as_ref()?;
        let Some((root, config_dir)) = &self.automatic_fixed_roots else {
            return Some(resolver);
        };
        let importer = match importing_file.canonicalize() {
            Ok(path) => normalize_path(&path),
            Err(_) => normalize_path(importing_file),
        };
        (importer.starts_with(root) || importer.starts_with(config_dir)).then_some(resolver)
    }
}

fn normalized_visible(visible: &ScopedHashSet<ScopedPathBuf>) -> ScopedHashSet<ScopedPathBuf> {
    visible
        .iter()
        .flat_map(|path| {
            let normalized = normalize_path(path);
            path.canonicalize().ok().map_or_else(
                || vec![normalized.clone()],
                |real| vec![normalized.clone(), normalize_path(&real)],
            )
        })
        .collect()
}
