use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "github-actions-step-timeouts";
const DEFAULT_INCLUDE: &[&str] = &[".github/workflows/**/*.yml", ".github/workflows/**/*.yaml"];
const GITHUB_MAX_MINUTES: u64 = 360;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    include: Vec<String>,
    max_minutes: Option<u64>,
    allow: Vec<AllowEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AllowEntry {
    step: String,
    reason: String,
    max_minutes: Option<u64>,
}

struct CompiledOptions {
    include: Vec<String>,
    max_minutes: u64,
    allow: BTreeMap<String, AllowEntry>,
    duplicate_allow: BTreeSet<String>,
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
    let mut allow = BTreeMap::new();
    let mut duplicate_allow = BTreeSet::new();
    for entry in opts.allow {
        let key = entry.step.clone();
        if allow.insert(key.clone(), entry).is_some() {
            duplicate_allow.insert(key);
        }
    }
    CompiledOptions {
        include: super::configured_includes(opts.include, DEFAULT_INCLUDE),
        max_minutes: opts
            .max_minutes
            .unwrap_or(GITHUB_MAX_MINUTES)
            .min(GITHUB_MAX_MINUTES),
        allow,
        duplicate_allow,
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

mod scan;
#[cfg(test)]
mod tests;
mod yaml;
