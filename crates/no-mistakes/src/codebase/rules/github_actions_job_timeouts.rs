use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod scan;
mod step_sum;
mod yaml;

pub const RULE_ID: &str = "github-actions-job-timeouts";

const DEFAULT_INCLUDE: &[&str] = &[".github/workflows/**/*.yml", ".github/workflows/**/*.yaml"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) max_minutes: Option<u64>,
    pub(crate) reject_step_exceeding_job: bool,
    pub(crate) reject_job_at_or_above_step_sum: bool,
    pub(crate) allow: Vec<AllowEntry>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct AllowEntry {
    pub(crate) job: String,
    pub(crate) max_minutes: Option<u64>,
}

struct CompiledOptions {
    include: Vec<String>,
    max_minutes: Option<u64>,
    reject_step_exceeding_job: bool,
    reject_job_at_or_above_step_sum: bool,
    allow: BTreeMap<String, Option<u64>>,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    super::check_with_borrowed_source_store(root, config, all_files, check_with_files_and_sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    super::run_selected_workflow_rule(
        super::SourcedRuleContext::new(root, config, all_files, sources),
        RULE_ID,
        compile_options,
        |opts| &opts.include,
        scan_files,
    )
}

fn compile_options(opts: Options) -> CompiledOptions {
    let include = super::configured_includes(opts.include, DEFAULT_INCLUDE);
    CompiledOptions {
        include,
        max_minutes: opts.max_minutes,
        reject_step_exceeding_job: opts.reject_step_exceeding_job,
        reject_job_at_or_above_step_sum: opts.reject_job_at_or_above_step_sum,
        allow: opts
            .allow
            .into_iter()
            .map(|entry| (entry.job, entry.max_minutes))
            .collect(),
    }
}

fn scan_files(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    files
        .par_iter()
        .flat_map(|path| scan::check_file(root, path, opts, sources))
        .collect()
}

#[cfg(test)]
mod tests;
