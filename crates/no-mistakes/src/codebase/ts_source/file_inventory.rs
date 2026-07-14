use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Stable identity for a lexical path in a frozen request file inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(u32);

impl FileId {
    /// Return the zero-based position of this file in its inventory.
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Deterministic, immutable file identities for one request-scoped path set.
///
/// Paths are normalized lexically, sorted, and deduplicated. Filesystem
/// canonicalization is intentionally avoided so a symlink and its target keep
/// distinct logical identities.
#[doc(hidden)]
pub struct FileInventory {
    paths: Arc<Vec<PathBuf>>,
}

impl FileInventory {
    #[doc(hidden)]
    pub fn from_paths(paths: &[PathBuf]) -> Self {
        let mut paths = paths
            .iter()
            .map(|path| super::normalize_discovery_path(path))
            .collect::<Vec<_>>();
        paths.sort();
        paths.dedup();

        assert!(
            u32::try_from(paths.len()).is_ok(),
            "request file inventory exceeds the FileId address space"
        );
        Self {
            paths: Arc::new(paths),
        }
    }

    #[doc(hidden)]
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    #[doc(hidden)]
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    #[doc(hidden)]
    pub fn paths(&self) -> Arc<Vec<PathBuf>> {
        Arc::clone(&self.paths)
    }

    #[doc(hidden)]
    pub fn id_for_path(&self, path: &Path) -> Option<FileId> {
        let normalized = super::normalize_discovery_path(path);
        self.paths
            .binary_search(&normalized)
            .ok()
            .map(|index| FileId(index as u32))
    }

    #[doc(hidden)]
    pub fn path(&self, id: FileId) -> Option<&Path> {
        self.paths.get(id.index()).map(PathBuf::as_path)
    }
}

#[cfg(test)]
mod tests;
