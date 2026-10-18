use super::AnalysisSession;
use crate::codebase::ts_resolver::{normalize_path, VisiblePathLookup, VisibleScopeIdentity};
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

mod canonical;
mod registry;

#[derive(Default)]
pub(super) struct VisibleScopes {
    first: OnceLock<Arc<PreparedVisible>>,
    scopes: OnceLock<registry::ScopeBuckets>,
    canonical: DashMap<canonical::CanonicalPath, Option<Arc<Path>>, rustc_hash::FxBuildHasher>,
}

/// Exact frozen projection. The registry retains its identity until the request
/// ends, so an address can never be recycled while a resolver key still uses it.
#[derive(Clone)]
pub(crate) struct PreparedVisible {
    paths: Arc<crate::fx::PathSet>,
    identity: VisibleScopeIdentity,
    normalized: Arc<OnceLock<Arc<crate::fx::PathSet>>>,
    standalone_cache: Arc<OnceLock<crate::codebase::ts_resolver::ImportResolutionCache>>,
}

impl PreparedVisible {
    pub(crate) fn new(paths: crate::fx::PathSet) -> Self {
        Self::from_shared(Arc::new(paths))
    }
    fn from_shared(paths: Arc<crate::fx::PathSet>) -> Self {
        Self {
            paths,
            identity: VisibleScopeIdentity::new(),
            normalized: Arc::default(),
            standalone_cache: Arc::default(),
        }
    }
    pub(crate) fn resolver_cache(&self) -> &crate::codebase::ts_resolver::ImportResolutionCache {
        self.standalone_cache.get_or_init(Default::default)
    }
}

impl FromIterator<PathBuf> for PreparedVisible {
    fn from_iter<T: IntoIterator<Item = PathBuf>>(paths: T) -> Self {
        Self::new(paths.into_iter().collect())
    }
}

impl std::ops::Deref for PreparedVisible {
    type Target = crate::fx::PathSet;
    fn deref(&self) -> &Self::Target {
        &self.paths
    }
}

impl VisiblePathLookup for PreparedVisible {
    fn contains_visible(&self, path: &Path) -> bool {
        self.paths.contains(path)
    }
    fn visible_len(&self) -> usize {
        self.paths.len()
    }
    fn visible_cache_key(&self) -> Vec<PathBuf> {
        self.paths.visible_cache_key()
    }
    fn visit_visible(&self, visit: &mut dyn FnMut(&Path)) {
        self.paths.visit_visible(visit);
    }
    fn scope_identity(&self) -> Option<VisibleScopeIdentity> {
        Some(self.identity.clone())
    }
    fn prepared_projection(&self) -> Option<PreparedVisible> {
        Some(self.clone())
    }
    fn normalized_visible(&self) -> Arc<crate::fx::PathSet> {
        Arc::clone(
            self.normalized
                .get_or_init(|| normalize_visible(&self.paths, canonical::alias)),
        )
    }
}

impl AnalysisSession {
    fn visible_scopes(&self) -> &VisibleScopes {
        self.visible_scopes.get_or_init(Default::default)
    }

    pub(crate) fn normalized_visible(
        &self,
        visible: &dyn VisiblePathLookup,
    ) -> Arc<PreparedVisible> {
        let scope = self.prepared_visible(visible);
        let registry = self.visible_scopes();
        let normalized = scope.normalized.get_or_init(|| {
            self.increment("resolver.visible_normalizations", 1);
            let inventories = canonical::inventories(self);
            normalize_visible(&scope.paths, |path, normalized| {
                if let Some(cached) = registry.canonical.get(path) {
                    cached.value().clone()
                } else {
                    registry
                        .canonical
                        .entry(canonical::CanonicalPath::new(path, &inventories))
                        .or_insert_with(|| {
                            self.increment("resolver.visible_canonicalize_calls", 1);
                            canonical::alias(path, normalized)
                        })
                        .value()
                        .clone()
                }
            })
        });
        if Arc::ptr_eq(normalized, &scope.paths) {
            return scope;
        }
        let prepared = self.prepared_visible(&PreparedVisible::from_shared(Arc::clone(normalized)));
        // This projection already includes all canonical aliases. Re-borrowing
        // it as prepared input must not canonicalize the aliases a second time.
        let _ = prepared.normalized.set(Arc::clone(&prepared.paths));
        prepared
    }
}

fn normalize_visible(
    paths: &Arc<crate::fx::PathSet>,
    mut canonical: impl FnMut(&Path, &Path) -> Option<Arc<Path>>,
) -> Arc<crate::fx::PathSet> {
    let mut output = None;
    for path in paths.iter() {
        let normalized = normalize_path(path);
        let alias = canonical(path, &normalized).filter(|path| !paths.contains(path.as_ref()));
        if normalized != *path || alias.is_some() {
            let output = output.get_or_insert_with(|| paths.as_ref().clone());
            if normalized != *path {
                output.remove(path);
                output.insert(normalized);
            }
            if let Some(alias) = alias {
                output.insert(alias.as_ref().to_path_buf());
            }
        }
    }
    output.map_or_else(|| Arc::clone(paths), Arc::new)
}

#[cfg(test)]
mod tests;
