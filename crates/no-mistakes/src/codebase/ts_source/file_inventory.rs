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

/// Discovery-time file classification for one lexical inventory path.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[doc(hidden)]
pub struct FileClassification {
    lexical_file: bool,
    lexical_symlink: bool,
    target_file: bool,
}

impl FileClassification {
    pub(crate) fn from_file_type(path: &Path, file_type: std::fs::FileType) -> Self {
        let lexical_file = file_type.is_file();
        let lexical_symlink = file_type.is_symlink();
        Self {
            lexical_file,
            lexical_symlink,
            target_file: lexical_file || (lexical_symlink && path.is_file()),
        }
    }

    #[doc(hidden)]
    pub fn is_lexical_file(self) -> bool {
        self.lexical_file
    }

    #[doc(hidden)]
    pub fn is_lexical_symlink(self) -> bool {
        self.lexical_symlink
    }

    #[doc(hidden)]
    pub fn target_is_file(self) -> bool {
        self.target_file
    }
}

#[derive(Debug)]
pub(crate) struct ClassifiedPath {
    pub(crate) path: PathBuf,
    pub(crate) classification: FileClassification,
}

/// Deterministic, immutable file identities for one request-scoped path set.
///
/// Paths are normalized lexically, sorted, and deduplicated. Filesystem
/// canonicalization is intentionally avoided so a symlink and its target keep
/// distinct logical identities.
#[doc(hidden)]
pub struct FileInventory {
    paths: Arc<Vec<PathBuf>>,
    classifications: Arc<Vec<FileClassification>>,
}

impl FileInventory {
    #[doc(hidden)]
    pub fn from_paths(paths: &[PathBuf]) -> Self {
        let paths = paths
            .iter()
            .map(|path| {
                let path = super::normalize_discovery_path(path);
                let classification = std::fs::symlink_metadata(&path)
                    .ok()
                    .map_or_else(FileClassification::default, |metadata| {
                        FileClassification::from_file_type(&path, metadata.file_type())
                    });
                ClassifiedPath {
                    path,
                    classification,
                }
            })
            .collect::<Vec<_>>();
        Self::from_classified_paths(paths)
    }

    pub(crate) fn from_classified_paths(mut entries: Vec<ClassifiedPath>) -> Self {
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        entries.dedup_by(|left, right| left.path == right.path);

        assert!(
            u32::try_from(entries.len()).is_ok(),
            "request file inventory exceeds the FileId address space"
        );
        Self {
            paths: Arc::new(entries.iter().map(|entry| entry.path.clone()).collect()),
            classifications: Arc::new(
                entries
                    .into_iter()
                    .map(|entry| entry.classification)
                    .collect(),
            ),
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
        if let Some(id) = self.id_for_normalized_path(path) {
            return Some(id);
        }
        let normalized = super::normalize_discovery_path(path);
        self.id_for_normalized_path(&normalized)
    }

    /// Look up a path that already crossed the discovery normalization
    /// boundary without allocating another `PathBuf`.
    #[doc(hidden)]
    pub fn id_for_normalized_path(&self, path: &Path) -> Option<FileId> {
        self.paths
            .binary_search_by(|candidate| candidate.as_path().cmp(path))
            .ok()
            .map(|index| FileId(index as u32))
    }

    #[doc(hidden)]
    pub fn path(&self, id: FileId) -> Option<&Path> {
        self.paths.get(id.index()).map(PathBuf::as_path)
    }

    #[doc(hidden)]
    pub fn classification(&self, id: FileId) -> Option<FileClassification> {
        self.classifications.get(id.index()).copied()
    }

    #[doc(hidden)]
    pub fn classification_for_path(&self, path: &Path) -> Option<FileClassification> {
        self.id_for_path(path)
            .and_then(|id| self.classification(id))
    }
}

#[cfg(test)]
mod tests;
