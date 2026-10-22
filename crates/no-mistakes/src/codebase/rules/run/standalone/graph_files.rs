use crate::codebase::rules::canonical_graph_requires_full_file_universe;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

pub(super) fn standalone_graph_files(
    root: &Path,
    config: &NoMistakesConfig,
    graph_plan: Option<crate::codebase::dependencies::graph::GraphBuildPlan>,
    visible_paths: &[PathBuf],
    scoped_files: &[PathBuf],
) -> Vec<PathBuf> {
    if canonical_graph_requires_full_file_universe(config) {
        crate::codebase::ts_source::discover_files_from_visible(root, &[], visible_paths)
    } else if graph_plan.is_some() {
        scoped_files.to_vec()
    } else {
        Vec::new()
    }
}
