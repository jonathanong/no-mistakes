use super::RuleFinding;
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

mod colocated_tests;
mod config;
mod coverage;
mod coverage_graph;
mod findings;
mod prepared;
mod runner;
mod selection;
mod types;

use colocated_tests::covered_components as colocated_test_covered_components;
use config::effective_story_patterns;
use coverage::{all_react_component_keys, directly_covered_components, reachable_story_files};
use coverage_graph::{dynamic_or_mock_boundary_files, transitive_covered_components};
use findings::{namespace_import_findings, stale_or_blank_allow_findings};
pub(crate) use prepared::{check_with_prepared_facts, check_with_prepared_facts_and_inferred};
use selection::{component_disabled, file_disabled, selected_components};
use types::{GlobMatcher, Options};

pub const RULE_ID: &str = "require-storybook-stories";

pub fn check(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    let files =
        crate::codebase::ts_source::discover_files(root, &config.filesystem.skip_directories);
    let facts = crate::codebase::check_facts::collect_check_facts(
        root,
        files,
        CheckFactPlan {
            react: true,
            symbols: true,
            storybook: true,
            dynamic_imports: true,
            source: true,
            ..Default::default()
        },
    );
    check_with_facts(root, config, tsconfig_path, &facts)
}

pub(crate) fn check_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
    shared: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    runner::check_with_tsconfig(
        root,
        config,
        shared,
        |project_root| {
            crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
                tsconfig_path,
                project_root,
                shared.files(),
            )
        },
        None,
    )
}

#[cfg(test)]
mod tests;
