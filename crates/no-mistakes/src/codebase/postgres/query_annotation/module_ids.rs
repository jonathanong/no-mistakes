use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// An immutable module identity, distinct from mutable evaluator frame IDs.
/// Assignment follows Path ordering so existing callback sort order is retained.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct ModuleId(usize);

#[derive(Default)]
pub(super) struct ModuleIds {
    paths: Vec<Arc<Path>>,
    indices: FxHashMap<Arc<Path>, ModuleId>,
    observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
}

impl PartialEq for ModuleIds {
    fn eq(&self, other: &Self) -> bool {
        self.paths == other.paths && self.indices == other.indices
    }
}
impl Eq for ModuleIds {}

impl ModuleIds {
    pub fn new(paths: impl IntoIterator<Item = PathBuf>) -> Self {
        let mut paths = paths.into_iter().collect::<Vec<_>>();
        paths.sort();
        paths.dedup();
        let paths = paths.into_iter().map(Arc::<Path>::from).collect::<Vec<_>>();
        let indices = paths
            .iter()
            .enumerate()
            .map(|(index, path)| (Arc::clone(path), ModuleId(index)))
            .collect();
        Self {
            paths,
            indices,
            observer: None,
        }
    }

    pub fn with_observer(
        mut self,
        observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
    ) -> Self {
        self.observer = observer.filter(|observer| observer.verbose());
        self
    }

    pub fn record_lookup(&self, hit: bool) {
        let Some(observer) = &self.observer else {
            return;
        };
        observer.increment(
            if hit {
                "postgres.annotation.module_cache_hits"
            } else {
                "postgres.annotation.module_cache_misses"
            },
            1,
        );
        if !hit {
            observer.increment("postgres.annotation.module_initializations", 1);
        }
    }

    pub fn id(&self, path: &Path) -> ModuleId {
        *self
            .indices
            .get(path)
            .expect("module path missing from the request's identity table")
    }

    pub fn path(&self, id: ModuleId) -> &Path {
        &self.paths[id.0]
    }
}

#[cfg(test)]
pub(super) mod tests;
