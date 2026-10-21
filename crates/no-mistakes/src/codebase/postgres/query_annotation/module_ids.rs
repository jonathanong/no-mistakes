use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// An immutable module identity, distinct from mutable evaluator frame IDs.
/// Assignment follows Path ordering so existing callback sort order is retained.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct ModuleId(usize);

#[derive(Default, Eq, PartialEq)]
pub(super) struct ModuleIds {
    paths: Vec<Arc<Path>>,
    indices: FxHashMap<Arc<Path>, ModuleId>,
}

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
        Self { paths, indices }
    }

    pub fn id(&self, path: &Path) -> ModuleId {
        self.indices[path]
    }

    pub fn path(&self, id: ModuleId) -> &Path {
        &self.paths[id.0]
    }
}

#[cfg(test)]
pub(super) mod tests;
