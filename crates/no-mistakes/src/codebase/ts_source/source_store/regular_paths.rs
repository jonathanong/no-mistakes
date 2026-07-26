use super::SourceStore;
use std::path::{Path, PathBuf};

impl SourceStore {
    /// Validate a lexical candidate once for suppression consumers. Regular
    /// inventory entries and registered snapshot paths need no containment
    /// fallback; symlinks and other supplemental paths do.
    pub(crate) fn validated_regular_path(&self, root: &Path, candidate: &Path) -> Option<PathBuf> {
        let normalized = super::super::normalize_discovery_path(candidate);
        if self
            .trusted_regular_paths
            .lock()
            .expect("trusted regular paths mutex poisoned")
            .contains(&normalized)
        {
            return Some(candidate.to_path_buf());
        }
        super::validation::validated_regular_path(
            &self.inventory,
            &self.validated_regular_paths,
            root,
            candidate,
        )
    }

    /// Trust exact regular files that came from an immutable discovery
    /// snapshot but sit outside this store's request-root inventory.
    pub(crate) fn register_trusted_regular_paths(&self, paths: &[PathBuf]) {
        let mut trusted = self
            .trusted_regular_paths
            .lock()
            .expect("trusted regular paths mutex poisoned");
        trusted.extend(paths.iter().filter_map(|path| {
            std::fs::symlink_metadata(path)
                .ok()
                .filter(|metadata| metadata.file_type().is_file())
                .map(|_| super::super::normalize_discovery_path(path))
        }));
    }

    pub(crate) fn trusted_regular_path(&self, candidate: &Path) -> Option<PathBuf> {
        let normalized = super::super::normalize_discovery_path(candidate);
        self.trusted_regular_paths
            .lock()
            .expect("trusted regular paths mutex poisoned")
            .contains(&normalized)
            .then(|| candidate.to_path_buf())
    }
}
