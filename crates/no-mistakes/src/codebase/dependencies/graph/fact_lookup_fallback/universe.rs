use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) fn same_graph_universe(
    primary_files: &[PathBuf],
    graph_visible: &dyn crate::codebase::ts_resolver::VisiblePathLookup,
) -> bool {
    let primary_visible: HashSet<&Path> = primary_files.iter().map(PathBuf::as_path).collect();
    if primary_visible.len() != graph_visible.visible_len() {
        return false;
    }
    let graph_paths = graph_visible.visible_cache_key();
    graph_paths
        .iter()
        .all(|path| primary_visible.contains(path.as_path()))
}
