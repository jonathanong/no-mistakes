use super::pipeline_text_setup::{
    build_route_import_graph_from_snapshot, load_route_import_tsconfig_from_snapshot,
};
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::codebase::ts_resolver::TsConfig;
use crate::playwright::config::Settings;
use crate::playwright::fsutil::VisiblePathSnapshot;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(crate) fn build_route_import_graph(
    root: &Path,
    settings: &Settings,
    facts: Option<&dyn TsFactLookup>,
    graph_file_universe: Option<&[PathBuf]>,
    route_source_files: &[PathBuf],
) -> Result<DepGraph> {
    let snapshot = VisiblePathSnapshot::new(root);
    build_route_import_graph_from_snapshot(
        root,
        settings,
        facts,
        graph_file_universe,
        route_source_files,
        &snapshot,
    )
}

pub(crate) fn load_route_import_tsconfig(root: &Path, settings: &Settings) -> Result<TsConfig> {
    let snapshot = VisiblePathSnapshot::new(root);
    load_route_import_tsconfig_from_snapshot(root, settings, &snapshot)
}
