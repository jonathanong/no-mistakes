use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Maps resolver results back into one frozen visible-path namespace.
///
/// Resolvers may return a real target below a symlink while facts retain the
/// lexical symlink path. Build this once per consumer so every lookup, cache
/// key, traversal, and rendered path uses the same request-local identity.
pub(crate) struct FrozenPathRemapper {
    visible: HashSet<PathBuf>,
    normalized_visible: Arc<HashSet<PathBuf>>,
    canonical_visible: HashMap<PathBuf, PathBuf>,
}

impl FrozenPathRemapper {
    pub(crate) fn from_paths(paths: impl IntoIterator<Item = PathBuf>) -> Self {
        let mut paths = paths.into_iter().collect::<Vec<_>>();
        paths.sort();
        paths.dedup();
        let visible = paths.iter().cloned().collect::<HashSet<_>>();
        let mut normalized_visible = HashSet::with_capacity(paths.len());
        let mut canonical_visible = HashMap::new();
        for path in &paths {
            normalized_visible.insert(crate::codebase::ts_resolver::normalize_path(path));
            let Some(canonical) = path.canonicalize().ok() else {
                continue;
            };
            let canonical = crate::codebase::ts_resolver::normalize_path(&canonical);
            normalized_visible.insert(canonical.clone());
            // Preserve a deterministic lexical graph namespace when a
            // symlink path and its real target are both visible.
            canonical_visible
                .entry(canonical)
                .or_insert_with(|| path.clone());
        }
        Self {
            visible,
            normalized_visible: Arc::new(normalized_visible),
            canonical_visible,
        }
    }

    /// The resolver's logical-and-canonical membership set. It is frozen in
    /// the same pass as the remapping table so consumers never canonicalize
    /// the full visible universe a second time.
    pub(crate) fn shared_normalized_visible(&self) -> Arc<HashSet<PathBuf>> {
        Arc::clone(&self.normalized_visible)
    }

    pub(crate) fn remap(&self, path: &Path) -> PathBuf {
        if let Some(path) = self.visible.get(path) {
            return path.clone();
        }
        let normalized = crate::codebase::ts_resolver::normalize_path(path);
        if let Some(path) = self.canonical_visible.get(&normalized) {
            return path.clone();
        }
        path.canonicalize()
            .ok()
            .map(|canonical| crate::codebase::ts_resolver::normalize_path(&canonical))
            .and_then(|canonical| self.canonical_visible.get(&canonical).cloned())
            .unwrap_or(normalized)
    }
}
