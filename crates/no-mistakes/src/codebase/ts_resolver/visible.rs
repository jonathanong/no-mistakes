/// Membership test for a frozen visible-path universe.
///
/// Graph construction uses [`crate::codebase::dependencies::graph::GraphFiles`]
/// so lookups binary-search interned paths instead of hashing cloned PathBufs.
/// HashSet remains for callers that already own a path set.
pub(crate) trait VisiblePathLookup: Send + Sync {
    fn contains_visible(&self, path: &Path) -> bool;

    fn visible_len(&self) -> usize;

    /// Paths that distinguish one resolver cache scope from another.
    fn visible_cache_key(&self) -> Vec<PathBuf>;
}

impl VisiblePathLookup for HashSet<PathBuf> {
    fn contains_visible(&self, path: &Path) -> bool {
        self.contains(path)
    }

    fn visible_len(&self) -> usize {
        self.len()
    }

    fn visible_cache_key(&self) -> Vec<PathBuf> {
        let mut paths: Vec<_> = self.iter().cloned().collect();
        paths.sort();
        paths
    }
}

impl VisiblePathLookup for crate::fx::PathSet {
    fn contains_visible(&self, path: &Path) -> bool {
        self.contains(path)
    }

    fn visible_len(&self) -> usize {
        self.len()
    }

    fn visible_cache_key(&self) -> Vec<PathBuf> {
        let mut paths: Vec<_> = self.iter().cloned().collect();
        paths.sort();
        paths
    }
}

impl<T: VisiblePathLookup + ?Sized> VisiblePathLookup for &T {
    fn contains_visible(&self, path: &Path) -> bool {
        (**self).contains_visible(path)
    }

    fn visible_len(&self) -> usize {
        (**self).visible_len()
    }

    fn visible_cache_key(&self) -> Vec<PathBuf> {
        (**self).visible_cache_key()
    }
}

impl<T: VisiblePathLookup + ?Sized> VisiblePathLookup for std::sync::Arc<T> {
    fn contains_visible(&self, path: &Path) -> bool {
        (**self).contains_visible(path)
    }

    fn visible_len(&self) -> usize {
        (**self).visible_len()
    }

    fn visible_cache_key(&self) -> Vec<PathBuf> {
        (**self).visible_cache_key()
    }
}

#[cfg(test)]
#[path = "visible_tests.rs"]
mod visible_tests;
