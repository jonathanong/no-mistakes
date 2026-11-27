pub(crate) mod ast;
mod checker;
pub(crate) mod config;
mod manual_mocks;
mod reachable;
mod runtime;
mod setup_helper_graph;
mod standalone;
mod with_facts;

use no_mistakes_rule_kernel::finding::RuleFinding;
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
pub use crate::codebase::testing_analysis::rule_ids::DYNAMIC_IMPORTS_RULE_ID as RULE_ID;

pub fn check(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    with_facts::check_standalone(root, config, tsconfig_path)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
