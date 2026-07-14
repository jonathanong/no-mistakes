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
    observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
}

struct SnapshotPathView {
    sources: Arc<SourceStore>,
    tracked_paths: Arc<Vec<PathBuf>>,
}

impl VisiblePathSnapshot {
    #[doc(hidden)]
    pub fn new(request_root: &Path) -> Self {
        Self::new_observed(request_root, None)
    }

    #[doc(hidden)]
    pub fn new_observed(
        request_root: &Path,
        observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
    ) -> Self {
        let normalized_request_root = normalize_discovery_path(request_root);
        let request_paths = discover_classified_path_views(&normalized_request_root);
        increment(&observer, "discovery.roots", 1);
        increment(
            &observer,
            "discovery.candidates",
            request_paths.visible.len() as u64,
        );
        let request_view = snapshot_path_view(request_paths, observer.clone());
        Self {
            request_root: normalized_request_root,
            request_view,
            scoped_views: Mutex::new(HashMap::new()),
            observer,
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
            request_view: snapshot_path_view_from_paths(request_paths, None),
            scoped_views: Mutex::new(HashMap::new()),
            observer: None,
        }
    }

    #[doc(hidden)]
    pub fn paths_for(&self, root: &Path) -> Arc<Vec<PathBuf>> {
        self.path_view_for(root).sources.inventory().paths()
    }

    /// Return the complete tracked path inventory for a discovered scope. In
    /// non-Git fallbacks, this is the complete ignore-aware visible path set.
    #[doc(hidden)]
    pub fn tracked_paths_for(&self, root: &Path) -> Arc<Vec<PathBuf>> {
        Arc::clone(&self.path_view_for(root).tracked_paths)
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
            match scoped_views.entry(normalized_root.clone()) {
                std::collections::hash_map::Entry::Occupied(entry) => {
                    increment(&self.observer, "discovery.cache_hits", 1);
                    Arc::clone(entry.get())
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    Arc::clone(entry.insert(Arc::new(OnceLock::new())))
                }
            }
        };
        Arc::clone(view.get_or_init(|| {
            if normalized_root.starts_with(&self.request_root)
                && !has_nested_git_boundary(&self.request_root, &normalized_root)
            {
                Arc::clone(&self.request_view)
            } else {
                let paths = discover_classified_path_views(&normalized_root);
                increment(&self.observer, "discovery.roots", 1);
                increment(
                    &self.observer,
                    "discovery.candidates",
                    paths.visible.len() as u64,
                );
                snapshot_path_view(paths, self.observer.clone())
            }
        }))
    }
}

fn snapshot_path_view(
    paths: DiscoveredClassifiedPathViews,
    observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
) -> Arc<SnapshotPathView> {
    let mut tracked_paths = paths
        .tracked
        .into_iter()
        .map(|path| normalize_discovery_path(&path))
        .collect::<Vec<_>>();
    tracked_paths.sort();
    tracked_paths.dedup();
    Arc::new(SnapshotPathView {
        sources: Arc::new(SourceStore::new_observed(
            Arc::new(FileInventory::from_classified_paths(paths.visible)),
            observer,
        )),
        tracked_paths: Arc::new(tracked_paths),
    })
}

fn snapshot_path_view_from_paths(
    paths: &[PathBuf],
    observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
) -> Arc<SnapshotPathView> {
    let inventory = Arc::new(FileInventory::from_paths(paths));
    let tracked_paths = inventory.paths();
    Arc::new(SnapshotPathView {
        sources: Arc::new(SourceStore::new_observed(inventory, observer)),
        tracked_paths,
    })
}

fn contains_path(paths: &[PathBuf], path: &Path) -> bool {
    paths
        .binary_search_by(|candidate| candidate.as_path().cmp(path))
        .is_ok()
}

fn increment(
    observer: &Option<Arc<crate::diagnostics::InvocationObserver>>,
    metric: &'static str,
    amount: u64,
) {
    if let Some(observer) = observer {
        observer.increment(metric, amount);
    }
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
