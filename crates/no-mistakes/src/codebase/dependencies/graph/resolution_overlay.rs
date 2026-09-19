use crate::codebase::ts_resolver::VisiblePathLookup;
use crate::codebase::ts_source::{cmp_os_str_paths, VisiblePathSnapshot};

/// Resolution membership for a bounded import walk.
///
/// `GraphFiles` may start as a candidate subset. The request snapshot is the
/// git-visible universe used to resolve escapes without admitting ignored files.
pub(crate) struct SnapshotResolutionVisible<'a> {
    graph_files: &'a GraphFiles,
    snapshot: &'a VisiblePathSnapshot,
    root: &'a Path,
}

impl<'a> SnapshotResolutionVisible<'a> {
    pub(crate) fn new(
        graph_files: &'a GraphFiles,
        snapshot: &'a VisiblePathSnapshot,
        root: &'a Path,
    ) -> Self {
        Self {
            graph_files,
            snapshot,
            root,
        }
    }
}

impl VisiblePathLookup for SnapshotResolutionVisible<'_> {
    fn contains_visible(&self, path: &Path) -> bool {
        if self.graph_files.contains_visible(path) {
            return true;
        }
        snapshot_contains_file(self.snapshot, self.root, path)
    }

    fn visible_len(&self) -> usize {
        self.snapshot.paths_for(self.root).len()
    }

    fn visible_cache_key(&self) -> Vec<PathBuf> {
        self.snapshot.paths_for(self.root).as_ref().clone()
    }
}

fn snapshot_contains_file(snapshot: &VisiblePathSnapshot, root: &Path, path: &Path) -> bool {
    let paths = snapshot.paths_for(root);
    if contains_sorted(&paths, path) {
        return path.is_file();
    }
    let normalized = crate::codebase::ts_resolver::normalize_path(path);
    contains_sorted(&paths, &normalized) && normalized.is_file()
}

fn contains_sorted(paths: &[PathBuf], path: &Path) -> bool {
    paths
        .binary_search_by(|candidate| cmp_os_str_paths(candidate, path))
        .is_ok()
}

pub(crate) struct ImportNeighborVisibility<'a> {
    pub(crate) graph_files: &'a GraphFiles,
    pub(crate) resolution_visible: Option<&'a dyn VisiblePathLookup>,
}

impl<'a> ImportNeighborVisibility<'a> {
    pub(crate) fn new(graph_files: &'a GraphFiles) -> Self {
        Self {
            graph_files,
            resolution_visible: None,
        }
    }

    fn lookup(&self) -> &dyn VisiblePathLookup {
        self.resolution_visible.unwrap_or(self.graph_files)
    }

    fn visible_path(&self, target: &Path) -> Option<PathBuf> {
        visible_or_escaped_path(self.graph_files, self.resolution_visible, target)
    }
}

pub(crate) fn visible_or_escaped_path(
    graph_files: &GraphFiles,
    resolution_visible: Option<&dyn VisiblePathLookup>,
    target: &Path,
) -> Option<PathBuf> {
    if let Some(path) = graph_files.visible_path(target) {
        return Some(path.to_path_buf());
    }
    let normalized = crate::codebase::ts_resolver::normalize_path(target);
    let visible = resolution_visible.is_some_and(|lookup| {
        lookup.contains_visible(target) || lookup.contains_visible(&normalized)
    });
    visible.then_some(normalized)
}
