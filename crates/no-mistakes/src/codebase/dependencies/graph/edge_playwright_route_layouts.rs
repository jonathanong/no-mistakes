use super::{Edge, EdgeKind, NodeId};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn collect_layout_chain_files_from_file_set(
    route_file: &Path,
    frontend_root: &Path,
    all_files: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    let mut layout_files = Vec::new();
    let mut current = route_file.parent();
    while let Some(parent) = current {
        if !parent.starts_with(frontend_root) {
            break;
        }
        for stem in ["layout", "loading", "error", "not-found", "template"] {
            for ext in ["tsx", "ts", "jsx", "js"] {
                let layout_file = parent.join(format!("{stem}.{ext}"));
                if all_files.contains(&layout_file) {
                    layout_files.push(layout_file);
                }
            }
        }
        current = parent.parent();
    }
    layout_files
}

pub(super) fn route_and_layout_edges(
    test_file: PathBuf,
    page_file: PathBuf,
    frontend_root: &Path,
    all_files: &HashSet<PathBuf>,
) -> Vec<Edge> {
    let mut edges = vec![(
        NodeId::File(test_file),
        NodeId::File(page_file.clone()),
        EdgeKind::RouteTest,
    )];
    edges.extend(
        collect_layout_chain_files_from_file_set(&page_file, frontend_root, all_files)
            .into_iter()
            .map(|layout_file| {
                (
                    NodeId::File(page_file.clone()),
                    NodeId::File(layout_file),
                    EdgeKind::Layout,
                )
            }),
    );
    edges
}
