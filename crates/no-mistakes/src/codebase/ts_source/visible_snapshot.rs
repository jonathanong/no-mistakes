use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Canonical, request-scoped view of paths that are not ignored.
///
/// The request root is discovered exactly once. Configured roots outside the
/// request root, and nested Git worktrees, receive their own bounded snapshot.
/// All state is in memory and is dropped with the request.
#[doc(hidden)]
pub struct VisiblePathSnapshot {
    request_root: PathBuf,
    request_sources: Arc<SourceStore>,
    scoped_sources: Mutex<HashMap<PathBuf, Arc<OnceLock<Arc<SourceStore>>>>>,
}

impl VisiblePathSnapshot {
    #[doc(hidden)]
    pub fn new(request_root: &Path) -> Self {
        let normalized_request_root = normalize_discovery_path(request_root);
        let request_sources = source_store(discover_visible_classified_paths(
            &normalized_request_root,
        ));
        Self {
            request_root: normalized_request_root,
            request_sources,
            scoped_sources: Mutex::new(HashMap::new()),
        }
    }

    /// Build a request snapshot from candidates already discovered by the
    /// caller. Graph requests use this to share their canonical file set with
    /// specialized collectors instead of starting a second repository scan.
    #[doc(hidden)]
    pub fn from_paths(request_root: &Path, request_paths: &[PathBuf]) -> Self {
        let normalized_request_root = normalize_discovery_path(request_root);
        Self {
            request_root: normalized_request_root,
            request_sources: Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
                request_paths,
            )))),
            scoped_sources: Mutex::new(HashMap::new()),
        }
    }

    #[doc(hidden)]
    pub fn paths_for(&self, root: &Path) -> Arc<Vec<PathBuf>> {
        self.source_store_for(root).inventory().paths()
    }

    #[doc(hidden)]
    pub fn classification_for(
        &self,
        root: &Path,
        path: &Path,
    ) -> Option<FileClassification> {
        self.source_store_for(root)
            .inventory()
            .classification_for_path(path)
    }

    /// Return the request-local source store backed by the same canonical file
    /// inventory as [`Self::paths_for`].
    #[doc(hidden)]
    pub fn source_store_for(&self, root: &Path) -> Arc<SourceStore> {
        if root == self.request_root {
            return Arc::clone(&self.request_sources);
        }
        let normalized_root = normalize_discovery_path(root);
        if normalized_root == self.request_root {
            return Arc::clone(&self.request_sources);
        }
        let sources = {
            let mut scoped_sources = self
                .scoped_sources
                .lock()
                .expect("visible-path snapshot mutex poisoned");
            Arc::clone(
                scoped_sources
                    .entry(normalized_root.clone())
                    .or_insert_with(|| Arc::new(OnceLock::new())),
            )
        };
        Arc::clone(sources.get_or_init(|| {
            if normalized_root.starts_with(&self.request_root)
                && !has_nested_git_boundary(&self.request_root, &normalized_root)
            {
                Arc::clone(&self.request_sources)
            } else {
                source_store(discover_visible_classified_paths(&normalized_root))
            }
        }))
    }
}

fn source_store(paths: Vec<ClassifiedPath>) -> Arc<SourceStore> {
    Arc::new(SourceStore::new(Arc::new(
        FileInventory::from_classified_paths(paths),
    )))
}

fn has_nested_git_boundary(request_root: &Path, root: &Path) -> bool {
    let mut current = root;
    while current != request_root {
        if current.join(".git").exists() {
            return true;
        }
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }
    false
}
