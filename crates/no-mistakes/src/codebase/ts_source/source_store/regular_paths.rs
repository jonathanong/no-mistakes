use super::SourceStore;
use std::path::{Path, PathBuf};

impl SourceStore {
    /// Validate a lexical candidate once for suppression consumers. Regular
    /// inventory entries and registered snapshot paths need no containment
    /// fallback; symlinks and other supplemental paths do.
    pub(crate) fn validated_regular_path(&self, root: &Path, candidate: &Path) -> Option<PathBuf> {
        let normalized = super::super::normalize_discovery_path(candidate);
        if self.trusted_regular_paths.contains(&normalized) {
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
    pub(crate) fn register_trusted_regular_paths(
        &self,
        paths: &[PathBuf],
        trusted_roots: &[PathBuf],
    ) {
        let canonical_roots = trusted_roots
            .iter()
            .filter_map(|root| std::fs::canonicalize(root).ok())
            .collect::<Vec<_>>();
        for path in paths {
            let Some(file_type) = std::fs::symlink_metadata(path)
                .ok()
                .map(|meta| meta.file_type())
            else {
                continue;
            };
            let trusted_file = file_type.is_file()
                || (file_type.is_symlink()
                    && std::fs::canonicalize(path).ok().is_some_and(|target| {
                        target.is_file()
                            && canonical_roots.iter().any(|root| target.starts_with(root))
                    }));
            if trusted_file {
                self.trusted_regular_paths
                    .insert(super::super::normalize_discovery_path(path));
            }
        }
    }

    pub(crate) fn trusted_regular_path(&self, candidate: &Path) -> Option<PathBuf> {
        let normalized = super::super::normalize_discovery_path(candidate);
        self.trusted_regular_paths
            .contains(&normalized)
            .then(|| candidate.to_path_buf())
    }
}
