use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod effort;
use super::github_actions_workflow_files::job_lines;
pub(super) use effort::partner_scope;
mod scan;
mod step_comparison;
mod step_sum;
mod yaml;

pub const RULE_ID: &str = "github-actions-job-timeouts";

pub(super) const DEFAULT_INCLUDE: &[&str] =
    &[".github/workflows/**/*.yml", ".github/workflows/**/*.yaml"];

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
    defer_suppression: bool,
    max_minutes: Option<u64>,
    reject_step_exceeding_job: bool,
    reject_job_at_or_above_step_sum: bool,
    allow: BTreeMap<String, Option<u64>>,
    partner_cap: Option<u64>,
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
    check_with_files_and_sources_deferred(root, config, all_files, sources, false)
}

pub(crate) fn check_with_files_and_sources_deferred(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    super::run_rule_applications(config, RULE_ID, |rule| {
        let raw: Options = rule.try_rule_options()?;
        let partner_cap = effort::certified_cap(config, rule)?;
        let (selected, roots) =
            super::selected_rule_files_with_roots(root, config, rule, all_files)?;
        let defaults = raw.include.is_empty();
        let matcher = super::file_matching::FileMatcher::new(
            root,
            &roots,
            raw.include
                .iter()
                .map(String::as_str)
                .chain(DEFAULT_INCLUDE.iter().copied().filter(|_| defaults)),
        )?;
        let missing = super::allow_audit::missing_file_findings(
            root,
            config,
            rule,
            sources,
            "allow",
            raw.allow.iter().map(|entry| {
                let file = entry.job.rsplit_once('#').map_or("", |(file, _)| file);
                let file = if matcher.is_match(&crate::codebase::ts_resolver::normalize_path(
                    &root.join(file),
                )) {
                    file.to_string()
                } else {
                    String::new()
                };
                (file, entry.job.clone())
            }),
        );
        let mut compiled = compile_options(raw);
        compiled.defer_suppression = defer_suppression;
        compiled.partner_cap = partner_cap;
        let files = matcher.select(&selected);
        let mut findings = scan_files(root, &compiled, &files, sources);
        findings.extend(missing);
        Ok(findings)
    })
}

fn compile_options(opts: Options) -> CompiledOptions {
    CompiledOptions {
        defer_suppression: false,
        max_minutes: opts.max_minutes,
        partner_cap: None,
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

#[cfg(test)]
mod effort_tests;

#[cfg(test)]
mod deferred_tests;
