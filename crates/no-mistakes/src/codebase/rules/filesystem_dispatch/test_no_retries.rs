mod options;
mod scan_ci;
mod scan_ci_lexer;
mod scan_ci_shell;
mod scan_ts;
mod scan_ts_calls;

use super::test_flake_helpers::{run_rule_applications, RunContext};
use crate::codebase::rules::github_actions_workflow_files::gha_kind;
use crate::codebase::rules::{self, RuleFinding};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "test-no-retries";

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
        options::compile,
        |compiled| compiled.files.as_ref(),
        scan_path,
    )
}

fn scan_path(
    root: &Path,
    path: &Path,
    opts: &options::CompiledOptions,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    if let Some(action) = gha_kind(&rel) {
        return scan_ci::scan_file(path, &rel, action, opts, sources);
    }
    scan_ts::scan_file(path, &rel, opts, sources)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
