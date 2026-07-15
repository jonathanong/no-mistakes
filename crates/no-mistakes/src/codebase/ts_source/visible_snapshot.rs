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
    request_view: Arc<SnapshotPathView>,
    scoped_views: Mutex<HashMap<PathBuf, Arc<OnceLock<Arc<SnapshotPathView>>>>>,
}

struct SnapshotPathView {
    sources: Arc<SourceStore>,
    tracked_paths: Arc<Vec<PathBuf>>,
}

impl VisiblePathSnapshot {
    #[doc(hidden)]
    pub fn new(request_root: &Path) -> Self {
        let normalized_request_root = normalize_discovery_path(request_root);
        let request_view =
            snapshot_path_view(discover_classified_path_views(&normalized_request_root));
        Self {
            request_root: normalized_request_root,
            request_view,
            scoped_views: Mutex::new(HashMap::new()),
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
            request_view: snapshot_path_view_from_paths(request_paths),
            scoped_views: Mutex::new(HashMap::new()),
        }
    }

    #[doc(hidden)]
    pub fn paths_for(&self, root: &Path) -> Arc<Vec<PathBuf>> {
        self.path_view_for(root).sources.inventory().paths()
    }

    /// Restrict candidates to the tracked (or non-Git fallback) inventories
    /// for request scopes that have already been discovered.
    #[doc(hidden)]
    pub fn tracked_paths_from(&self, candidates: &[PathBuf]) -> Vec<PathBuf> {
        let scoped_views = self
            .scoped_views
            .lock()
            .expect("visible-path snapshot mutex poisoned");
        candidates
            .iter()
            .map(|path| normalize_discovery_path(path))
            .filter(|path| {
                contains_path(&self.request_view.tracked_paths, path)
                    || scoped_views
                        .values()
                        .filter_map(|view| view.get())
                        .any(|view| contains_path(&view.tracked_paths, path))
            })
            .collect()
    }

    #[doc(hidden)]
    pub fn classification_for(&self, root: &Path, path: &Path) -> Option<FileClassification> {
        self.source_store_for(root)
            .inventory()
            .classification_for_path(path)
    }

    /// Return the request-local source store backed by the same canonical file
    /// inventory as [`Self::paths_for`].
    #[doc(hidden)]
    pub fn source_store_for(&self, root: &Path) -> Arc<SourceStore> {
        Arc::clone(&self.path_view_for(root).sources)
    }

    fn path_view_for(&self, root: &Path) -> Arc<SnapshotPathView> {
        if root == self.request_root {
            return Arc::clone(&self.request_view);
        }
        let normalized_root = normalize_discovery_path(root);
        if normalized_root == self.request_root {
            return Arc::clone(&self.request_view);
        }
        let view = {
            let mut scoped_views = self
                .scoped_views
                .lock()
                .expect("visible-path snapshot mutex poisoned");
            Arc::clone(
                scoped_views
                    .entry(normalized_root.clone())
                    .or_insert_with(|| Arc::new(OnceLock::new())),
            )
        };
        Arc::clone(view.get_or_init(|| {
            if normalized_root.starts_with(&self.request_root)
                && !has_nested_git_boundary(&self.request_root, &normalized_root)
            {
                Arc::clone(&self.request_view)
            } else {
                snapshot_path_view(discover_classified_path_views(&normalized_root))
            }
        }))
    }
}

fn snapshot_path_view(paths: DiscoveredClassifiedPathViews) -> Arc<SnapshotPathView> {
    let mut tracked_paths = paths
        .tracked
        .into_iter()
        .map(|path| normalize_discovery_path(&path))
        .collect::<Vec<_>>();
    tracked_paths.sort();
    tracked_paths.dedup();
    Arc::new(SnapshotPathView {
        sources: Arc::new(SourceStore::new(Arc::new(
            FileInventory::from_classified_paths(paths.visible),
        ))),
        tracked_paths: Arc::new(tracked_paths),
    })
}

fn snapshot_path_view_from_paths(paths: &[PathBuf]) -> Arc<SnapshotPathView> {
    let inventory = Arc::new(FileInventory::from_paths(paths));
    let tracked_paths = inventory.paths();
    Arc::new(SnapshotPathView {
        sources: Arc::new(SourceStore::new(inventory)),
        tracked_paths,
    })
}

fn contains_path(paths: &[PathBuf], path: &Path) -> bool {
    paths
        .binary_search_by(|candidate| candidate.as_path().cmp(path))
        .is_ok()
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
