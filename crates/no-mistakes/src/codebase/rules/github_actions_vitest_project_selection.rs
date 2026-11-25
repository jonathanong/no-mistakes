use super::RuleFinding;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod command;
mod judge;
pub(in crate::codebase::rules) mod options;
mod runs;

pub const RULE_ID: &str = "github-actions-vitest-project-selection";

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Options {
    pub(crate) projects: Vec<ProjectOption>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ProjectOption {
    pub(crate) project: String,
    pub(crate) workflows: Vec<String>,
    pub(crate) name_filters: Option<String>,
}

super::sourced_file_rule_entrypoints!();

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let compiled = options::compile(opts)?;
    if compiled.projects.is_empty() {
        return Ok(Vec::new());
    }
    super::github_actions_workflow_files::scan_files(root, files, sources, &|rel, content| {
        runs::check_source(rel, content, &compiled)
    })
}

#[cfg(test)]
mod tests;
