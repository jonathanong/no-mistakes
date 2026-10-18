use super::*;
use std::hash::{Hash, Hasher};

pub(super) type ScopeBuckets =
    DashMap<(usize, u64), Vec<Arc<PreparedVisible>>, rustc_hash::FxBuildHasher>;

impl VisibleScopes {
    pub(super) fn buckets(&self) -> &ScopeBuckets {
        self.scopes.get_or_init(Default::default)
    }
}

impl AnalysisSession {
    pub(crate) fn prepared_visible(&self, visible: &dyn VisiblePathLookup) -> Arc<PreparedVisible> {
        let registry = self.visible_scopes();
        let mut created = false;
        let first = registry.first.get_or_init(|| {
            created = true;
            collect(visible)
        });
        // The common single-projection path needs no content fingerprint,
        // bucket table, or second membership walk after collecting its input.
        if created || same_projection(visible, first) {
            return Arc::clone(first);
        }
        let mut bucket = registry
            .buckets()
            .entry(visible_fingerprint(visible))
            .or_default();
        for scope in bucket.iter() {
            if same_projection(visible, scope) {
                return Arc::clone(scope);
            }
        }
        let scope = collect(visible);
        bucket.push(Arc::clone(&scope));
        scope
    }
}

fn collect(visible: &dyn VisiblePathLookup) -> Arc<PreparedVisible> {
    let mut paths = crate::fx::PathSet::default();
    visible.visit_visible(&mut |path| {
        paths.insert(path.to_path_buf());
    });
    Arc::new(PreparedVisible::new(paths))
}

fn same_projection(visible: &dyn VisiblePathLookup, scope: &PreparedVisible) -> bool {
    if visible
        .scope_identity()
        .is_some_and(|id| id == scope.identity)
    {
        return true;
    }
    if visible.visible_len() != scope.paths.len() {
        return false;
    }
    // A fingerprint only selects a bucket. Full membership prevents a collision
    // from changing an unresolved import into a wrong known target.
    let mut equal = true;
    visible.visit_visible(&mut |path| equal &= scope.paths.contains(path));
    equal
}

pub(super) fn visible_fingerprint(visible: &dyn VisiblePathLookup) -> (usize, u64) {
    let mut fingerprint = 0u64;
    visible.visit_visible(&mut |path| {
        let mut hash = rustc_hash::FxHasher::default();
        path.hash(&mut hash);
        fingerprint = fingerprint.wrapping_add(hash.finish());
    });
    (visible.visible_len(), fingerprint)
}
