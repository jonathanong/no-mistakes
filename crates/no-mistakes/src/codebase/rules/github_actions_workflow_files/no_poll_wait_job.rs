use crate::codebase::rules::{self, RuleFinding};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod loops;
mod runs;

pub const RULE_ID: &str = "github-actions-no-poll-wait-job";

/// `pollCommands` is empty by default, so the rule reports nothing until a
/// configuration names the commands that poll another job or workflow.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    poll_commands: Vec<String>,
    ignore_for_loops: bool,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    rules::check_with_borrowed_source_store(root, config, all_files, check_with_files_and_sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    rules::run_sourced_file_rule(root, config, all_files, sources, RULE_ID, scan)
}

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    let polls = compile(opts)?;
    if polls.is_empty() {
        return Ok(Vec::new());
    }
    let skip_for = opts.ignore_for_loops;
    super::scan_files(root, files, sources, &|rel, content| {
        runs::check_source(rel, content, &polls, skip_for)
    })
}

fn compile(opts: &Options) -> Result<Vec<Vec<String>>> {
    opts.poll_commands
        .iter()
        .map(|command| {
            let words: Vec<String> = command.split_whitespace().map(str::to_string).collect();
            if words.is_empty() {
                anyhow::bail!("{RULE_ID} pollCommands entries must name a command");
            }
            Ok(words)
        })
        .collect()
}

#[cfg(test)]
mod tests;
