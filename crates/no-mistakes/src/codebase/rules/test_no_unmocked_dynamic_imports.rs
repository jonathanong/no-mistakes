pub(crate) mod ast;
mod checker;
pub(crate) mod config;
mod manual_mocks;
mod reachable;
mod runtime;
mod standalone;
mod with_facts;

use super::RuleFinding;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::ts_source::discover_files;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
pub(crate) use runtime::runtime_deps;
pub(crate) use standalone::{
    check_inner, matching_test_files_with_filter, remap_resolved_path, resolve_mock_specifiers,
};
use std::path::Path;
pub(crate) use with_facts::check_with_prepared_facts_graph_and_session;
pub use with_facts::{check_with_facts, check_with_prepared_facts};

pub const RULE_ID: &str = "test-no-unmocked-dynamic-imports";

pub fn check(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    let files = discover_files(root, &config.filesystem.skip_directories);
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(tsconfig_path, root, &files)?;
    let graph_files = crate::codebase::dependencies::graph::GraphFiles::from_files(files.clone());
    let graph = DepGraph::build_with_plan_and_files(
        root,
        &tsconfig,
        GraphBuildPlan::imports_and_workspace(),
        &graph_files,
    )?;
    let manual_mocks = manual_mocks::discover_from_files(root, &files);
    check_inner(root, config, &files, &tsconfig, &graph, &manual_mocks)
}

#[cfg(test)]
mod tests;
