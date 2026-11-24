mod scan;

use super::test_flake_helpers::{
    compile_optional_globs, configured_paths, is_playwright_config_name, run_rule_applications,
    AllowEntry, CompiledAllowList, RunContext,
};
use crate::codebase::rules::path_filter::GlobMatcher;
use crate::codebase::rules::{self, RuleFinding};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "playwright-no-serial";

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    allow: Vec<AllowEntry>,
    files: Vec<String>,
    configs: Vec<String>,
}

struct CompiledOptions {
    allow: CompiledAllowList,
    files: Option<GlobMatcher>,
    configs: Option<GlobMatcher>,
    runner_configs: BTreeSet<String>,
}

impl CompiledOptions {
    fn is_playwright_config(&self, rel: &str) -> bool {
        self.runner_configs.contains(rel)
            || self
                .configs
                .as_ref()
                .is_some_and(|matcher| matcher.is_match(rel))
            || is_playwright_config_name(rel)
    }
}

fn compile_options(opts: Options, config: &NoMistakesConfig) -> Result<CompiledOptions> {
    Ok(CompiledOptions {
        allow: CompiledAllowList::compile(RULE_ID, &opts.allow)?,
        files: compile_optional_globs(&opts.files, &format!("{RULE_ID} files"))?,
        configs: compile_optional_globs(&opts.configs, &format!("{RULE_ID} configs"))?,
        runner_configs: configured_paths(&config.tests.playwright.configs),
    })
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    rules::check_sourced_standalone(root, config, check_with_files)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = rules::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources, false)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    run_rule_applications(
        RunContext {
            rule_id: RULE_ID,
            root,
            config,
            all_files,
            sources,
            defer_suppression,
        },
        compile_options,
        |compiled| compiled.files.as_ref(),
        scan::scan_file,
    )
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
