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
        self.graph_files.contains_visible(path)
            || snapshot_visible_path(self.snapshot, self.root, path).is_some()
    }

    fn visible_len(&self) -> usize {
        self.snapshot.paths_for(self.root).len()
    }

    fn visible_cache_key(&self) -> Vec<PathBuf> {
        let mut paths = self.snapshot.paths_for(self.root).as_ref().clone();
        paths.extend(self.graph_files.iter_visible().cloned());
        crate::codebase::ts_source::sort_os_str_paths(&mut paths);
        paths.dedup();
        paths
    }

    fn visible_alias(&self, path: &Path) -> Option<PathBuf> {
        self.graph_files
            .visible_path(path)
            .map(Path::to_path_buf)
            .or_else(|| snapshot_visible_path(self.snapshot, self.root, path))
    }
}

fn snapshot_visible_path(
    snapshot: &VisiblePathSnapshot,
    root: &Path,
    path: &Path,
) -> Option<PathBuf> {
    let paths = snapshot.paths_for(root);
    if let Some(hit) = visible_sorted_file(&paths, path) {
        return Some(hit);
    }
    let normalized = crate::codebase::ts_resolver::normalize_path(path);
    if let Some(hit) = visible_sorted_file(&paths, &normalized) {
        return Some(hit);
    }
    let canonical = crate::codebase::ts_resolver::normalize_path(&path.canonicalize().ok()?);
    if let Some(hit) = visible_sorted_file(&paths, &canonical) {
        return Some(hit);
    }
    let real_root = crate::codebase::ts_resolver::normalize_path(&root.canonicalize().ok()?);
    let relative = canonical.strip_prefix(&real_root).ok()?;
    let lexical = crate::codebase::ts_resolver::normalize_path(&root.join(relative));
    visible_sorted_file(&paths, &lexical)
}

fn visible_sorted_file(paths: &[PathBuf], path: &Path) -> Option<PathBuf> {
    contains_sorted(paths, path)
        .then(|| path.to_path_buf())
        .filter(|hit| hit.is_file())
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
    pub(crate) fn new(
        graph_files: &'a GraphFiles,
        resolution_visible: Option<&'a dyn VisiblePathLookup>,
    ) -> Self {
        Self {
            graph_files,
            resolution_visible,
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
    let lookup = resolution_visible?;
    lookup
        .visible_alias(target)
        .or_else(|| lookup.visible_alias(&crate::codebase::ts_resolver::normalize_path(target)))
}
