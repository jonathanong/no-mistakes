use super::AnalysisSession;
pub(crate) use crate::codebase::ts_resolver::PreparedVisible;
use crate::codebase::ts_resolver::{canonical_alias, normalize_visible_paths, VisiblePathLookup};
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
        let normalized = scope.normalized_cell().get_or_init(|| {
            self.increment("resolver.visible_normalizations", 1);
            let inventories = canonical::inventories(self);
            normalize_visible_paths(scope.paths_arc(), |path, normalized| {
                if let Some(cached) = registry.canonical.get(path) {
                    cached.value().clone()
                } else {
                    registry
                        .canonical
                        .entry(canonical::CanonicalPath::new(path, &inventories))
                        .or_insert_with(|| {
                            self.increment("resolver.visible_canonicalize_calls", 1);
                            canonical_alias(path, normalized)
                        })
                        .value()
                        .clone()
                }
            })
        });
        if Arc::ptr_eq(normalized, scope.paths_arc()) {
            return scope;
        }
        let prepared = self.prepared_visible(&PreparedVisible::from_shared(Arc::clone(normalized)));
        // This projection already includes all canonical aliases. Re-borrowing
        // it as prepared input must not canonicalize the aliases a second time.
        let _ = prepared
            .normalized_cell()
            .set(Arc::clone(prepared.paths_arc()));
        prepared
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
