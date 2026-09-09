use super::super::roots;
use super::Options;
use std::path::{Path, PathBuf};

pub(crate) fn expand_roots(
    root: &Path,
    options: &Options,
    graph: &crate::codebase::dependencies::graph::DepGraph,
    files: &[PathBuf],
) -> anyhow::Result<Vec<crate::codebase::dependencies::graph::NodeId>> {
    roots::expand(roots::ExpandRequest {
        root,
        options,
        graph,
        vitest: None,
        playwright: None,
        graph_files: files,
        target_roots: &[],
    })
}
