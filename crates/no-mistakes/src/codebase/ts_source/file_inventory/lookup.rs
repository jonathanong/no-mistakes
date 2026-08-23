use super::{ClassifiedPath, FileClassification, FileInventory};
use std::path::PathBuf;

impl FileInventory {
    pub(crate) fn as_paths(&self) -> &[PathBuf] {
        self.paths.as_slice()
    }

    /// Build an identity map from already-normalized keys without filesystem
    /// classification. Collectors that already hold a request inventory should
    /// reuse that inventory instead of calling this.
    pub(crate) fn from_lookup_paths(paths: impl IntoIterator<Item = PathBuf>) -> Self {
        Self::from_classified_paths(
            paths
                .into_iter()
                .map(|path| ClassifiedPath {
                    path,
                    classification: FileClassification::default(),
                })
                .collect(),
        )
    }
}
