pub(crate) mod ast;
mod checker;
pub(crate) mod config;
mod manual_mocks;
mod reachable;
mod runtime;
mod setup_helper_graph;
mod standalone;
mod with_facts;

use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
pub(crate) use runtime::runtime_deps;
pub(crate) use standalone::{
    matching_test_files_with_filter, remap_resolved_path, resolve_mock_specifiers,
};
use std::path::Path;
pub use with_facts::{check_with_facts, check_with_prepared_facts};
pub(crate) use with_facts::{
    check_with_prepared_facts_graph_and_session_with_suppression, supplement_runner_facts,
    PreparedFactsGraphRequest,
};
pub const RULE_ID: &str = "test-no-unmocked-dynamic-imports";

pub fn check(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    with_facts::check_standalone(root, config, tsconfig_path)
}

#[cfg(test)]
mod tests;
