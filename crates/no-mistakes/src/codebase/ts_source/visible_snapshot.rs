use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

include!("visible_snapshot_supplied.rs");
include!("visible_snapshot_paths.rs");
include!("visible_snapshot_scopes.rs");

/// Canonical, request-scoped view of paths that are not ignored.
///
/// The request root is discovered exactly once. Configured roots outside the
/// request root, and nested Git worktrees, receive their own bounded snapshot.
/// All state is in memory and is dropped with the request.
#[doc(hidden)]
pub struct VisiblePathSnapshot {
    request_root: PathBuf,
    authoritative_tracked_paths: bool,
    request_view: Arc<SnapshotPathView>,
    scoped_views: Mutex<HashMap<PathBuf, Arc<OnceLock<Arc<SnapshotPathView>>>>>,
    scope_roots: Mutex<HashMap<PathBuf, PathBuf>>,
    projected_paths: Mutex<HashMap<PathBuf, Arc<OnceLock<ProjectedSnapshotPaths>>>>,
    observer: Option<Arc<crate::diagnostics::InvocationObserver>>,
}

struct SnapshotPathView {
    sources: Arc<SourceStore>,
    tracked_paths: Arc<Vec<PathBuf>>,
    git_index_paths: Arc<Vec<PathBuf>>,
    git_index_available: bool,
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
            authoritative_tracked_paths: false,
            request_view,
            scoped_views: Mutex::new(HashMap::new()),
            scope_roots: Mutex::new(HashMap::new()),
            projected_paths: Mutex::new(HashMap::new()),
            observer,
        }
    }

    #[doc(hidden)]
    pub fn paths_for(&self, root: &Path) -> Arc<Vec<PathBuf>> {
        let view = self.path_view_for(root);
        self.project_paths(root, &view, SnapshotPathKind::Visible)
    }

    /// Return the worktree-readable tracked path inventory for a scope. In
    /// non-Git fallbacks, this is the complete ignore-aware visible path set.
    #[doc(hidden)]
    pub fn tracked_paths_for(&self, root: &Path) -> Arc<Vec<PathBuf>> {
        let view = self.path_view_for(root);
        self.project_paths(root, &view, SnapshotPathKind::Tracked)
    }

    /// Whether this prepared scope can prove tracked membership.
    /// Generic supplied path lists and non-Git fallbacks cannot prove tracked
    /// membership; explicitly authoritative tracked lists can.
    #[doc(hidden)]
    pub fn git_index_available_for(&self, root: &Path) -> bool {
        self.path_view_for(root).git_index_available
    }

    /// Restrict candidates to the tracked (or non-Git fallback) inventories
    /// for request scopes that have already been discovered.
    #[doc(hidden)]
    pub fn tracked_paths_from(&self, candidates: &[PathBuf]) -> Vec<PathBuf> {
        let scoped_views = self
            .scoped_views
            .lock()
            .expect("visible-path snapshot mutex poisoned");
        let mut tracked = Vec::new();
        for candidate in candidates {
            let path = normalize_discovery_path(candidate);
            if contains_path(&self.request_view.tracked_paths, &path) {
                tracked.push(path);
                continue;
            }
            for view in scoped_views.values() {
                let Some(view) = view.get() else {
                    continue;
                };
                if contains_path(&view.tracked_paths, &path) {
                    tracked.push(path);
                    break;
                }
            }
        }
        tracked
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
        if self.authoritative_tracked_paths || root == self.request_root {
            return Arc::clone(&self.request_view);
        }
        let normalized_root = normalize_discovery_path(root);
        if normalized_root == self.request_root {
            return Arc::clone(&self.request_view);
        }
        let scope_root = self.cached_scope_root(&normalized_root);
        if scope_root == self.request_root {
            return Arc::clone(&self.request_view);
        }
        let view = {
            let mut scoped_views = self
                .scoped_views
                .lock()
                .expect("visible-path snapshot mutex poisoned");
            match scoped_views.entry(scope_root.clone()) {
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
            let paths = discover_classified_path_views(&scope_root);
            increment(&self.observer, "discovery.roots", 1);
            increment(
                &self.observer,
                "discovery.candidates",
                paths.visible.len() as u64,
            );
            snapshot_path_view(paths, self.observer.clone())
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
    sort_os_str_paths(&mut tracked_paths);
    tracked_paths.dedup();
    Arc::new(SnapshotPathView {
        sources: Arc::new(SourceStore::new_observed(
            Arc::new(FileInventory::from_classified_paths_counted(
                paths.visible,
                paths.metadata_stats,
            )),
            observer,
        )),
        tracked_paths: Arc::new(tracked_paths),
        git_index_paths: normalized_index_paths(&paths.git_index_paths),
        git_index_available: paths.git_index_available,
    })
}


fn contains_path(paths: &[PathBuf], path: &Path) -> bool {
    // Exact OsStr membership. Canonical remapping does not belong here.
    paths
        .binary_search_by(|candidate| cmp_os_str_paths(candidate, path))
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
