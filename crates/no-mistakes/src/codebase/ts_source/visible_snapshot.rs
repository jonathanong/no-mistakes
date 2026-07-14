use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Canonical, request-scoped view of paths that are not ignored.
///
/// The request root is discovered exactly once. Configured roots outside the
/// request root, and nested Git worktrees, receive their own bounded snapshot.
/// All state is in memory and is dropped with the request.
#[doc(hidden)]
pub struct VisiblePathSnapshot {
    request_root: PathBuf,
    request_paths: Arc<Vec<PathBuf>>,
    additional_roots: Mutex<HashMap<PathBuf, Arc<Vec<PathBuf>>>>,
}

impl VisiblePathSnapshot {
    #[doc(hidden)]
    pub fn new(request_root: &Path) -> Self {
        let normalized_request_root = normalize_discovery_path(request_root);
        let request_paths = discover_visible_paths(&normalized_request_root);
        Self {
            request_root: normalized_request_root,
            request_paths: Arc::new(request_paths),
            additional_roots: Mutex::new(HashMap::new()),
        }
    }

    /// Build a request snapshot from candidates already discovered by the
    /// caller. Graph requests use this to share their canonical file set with
    /// specialized collectors instead of starting a second repository scan.
    #[doc(hidden)]
    pub fn from_paths(request_root: &Path, request_paths: &[PathBuf]) -> Self {
        let normalized_request_root = normalize_discovery_path(request_root);
        let mut request_paths = request_paths
            .iter()
            .map(|path| normalize_discovery_path(path))
            .collect::<Vec<_>>();
        request_paths.sort();
        request_paths.dedup();
        Self {
            request_root: normalized_request_root,
            request_paths: Arc::new(request_paths),
            additional_roots: Mutex::new(HashMap::new()),
        }
    }

    #[doc(hidden)]
    pub fn paths_for(&self, root: &Path) -> Arc<Vec<PathBuf>> {
        let normalized_root = normalize_discovery_path(root);
        if (normalized_root == self.request_root || normalized_root.starts_with(&self.request_root))
            && !has_nested_git_boundary(&self.request_root, &normalized_root)
        {
            return Arc::clone(&self.request_paths);
        }
        let mut additional_roots = self
            .additional_roots
            .lock()
            .expect("visible-path snapshot mutex poisoned");
        Arc::clone(
            additional_roots
                .entry(normalized_root.clone())
                .or_insert_with(|| Arc::new(discover_visible_paths(&normalized_root))),
        )
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
