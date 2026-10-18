use super::AnalysisSession;
use crate::codebase::ts_resolver::{normalize_path, VisiblePathLookup, VisibleScopeIdentity};
use dashmap::DashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

#[derive(Default)]
pub(super) struct VisibleScopes {
    scopes: DashMap<(usize, u64), Vec<Arc<PreparedVisible>>, rustc_hash::FxBuildHasher>,
    canonical: DashMap<PathBuf, Option<Arc<Path>>, rustc_hash::FxBuildHasher>,
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
        Self {
            paths: Arc::new(paths),
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
    fn normalized_visible(&self) -> Arc<crate::fx::PathSet> {
        Arc::clone(self.normalized.get_or_init(|| {
            Arc::new(crate::codebase::ts_resolver::normalized_visible_path_set(
                self.paths.iter().cloned(),
            ))
        }))
    }
}

impl AnalysisSession {
    fn visible_scopes(&self) -> &VisibleScopes {
        self.visible_scopes.get_or_init(Default::default)
    }

    pub(crate) fn prepared_visible(&self, visible: &dyn VisiblePathLookup) -> Arc<PreparedVisible> {
        let mut bucket = self
            .visible_scopes()
            .scopes
            .entry(visible_fingerprint(visible))
            .or_default();
        for scope in bucket.iter() {
            let mut equal = true;
            visible.visit_visible(&mut |path| equal &= scope.paths.contains(path));
            if equal {
                return Arc::clone(scope);
            }
        }
        // The fingerprint selects a bucket only. Compare full membership above
        // so even a hash collision cannot return a wrong known target.
        let mut paths = crate::fx::PathSet::default();
        visible.visit_visible(&mut |path| {
            paths.insert(path.to_path_buf());
        });
        let scope = Arc::new(PreparedVisible::new(paths));
        bucket.push(Arc::clone(&scope));
        scope
    }

    pub(crate) fn normalized_visible(
        &self,
        visible: &dyn VisiblePathLookup,
    ) -> Arc<PreparedVisible> {
        let scope = self.prepared_visible(visible);
        let registry = self.visible_scopes();
        let normalized = scope.normalized.get_or_init(|| {
            self.increment("resolver.visible_normalizations", 1);
            let mut paths = crate::fx::PathSet::default();
            for path in scope.paths.iter() {
                let normalized = normalize_path(path);
                let canonical = if let Some(cached) = registry.canonical.get(path) {
                    cached.value().clone()
                } else {
                    registry
                        .canonical
                        .entry(path.clone())
                        .or_insert_with(|| {
                            self.increment("resolver.visible_canonicalize_calls", 1);
                            // Identity spellings and failures both need no extra
                            // alias. Retain text only when canonicalization adds one.
                            path.canonicalize()
                                .ok()
                                .map(|path| normalize_path(&path))
                                .filter(|real| real != &normalized)
                                .map(Arc::<Path>::from)
                        })
                        .value()
                        .clone()
                };
                paths.insert(normalized);
                if let Some(path) = canonical {
                    paths.insert(path.as_ref().to_path_buf());
                }
            }
            if paths == *scope.paths {
                Arc::clone(&scope.paths)
            } else {
                Arc::new(paths)
            }
        });
        if normalized.as_ref() == scope.paths.as_ref() {
            return scope;
        }
        let prepared = self.prepared_visible(normalized);
        // This projection already includes all canonical aliases. Re-borrowing
        // it as prepared input must not canonicalize the aliases a second time.
        let _ = prepared.normalized.set(Arc::clone(&prepared.paths));
        prepared
    }
}

fn visible_fingerprint(visible: &dyn VisiblePathLookup) -> (usize, u64) {
    let mut fingerprint = 0u64;
    visible.visit_visible(&mut |path| {
        let mut hash = rustc_hash::FxHasher::default();
        path.hash(&mut hash);
        fingerprint = fingerprint.wrapping_add(hash.finish());
    });
    (visible.visible_len(), fingerprint)
}

#[cfg(test)]
mod tests;
