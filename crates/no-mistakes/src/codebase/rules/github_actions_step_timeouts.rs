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
    defer_suppression: bool,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources, false)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    super::run_rule_applications(config, RULE_ID, |rule| {
        let mut compiled = compile_options(rule.try_rule_options()?);
        compiled.defer_suppression = defer_suppression;
        let (selected, target_roots) =
            super::selected_rule_files_with_roots(root, config, rule, all_files)?;
        let files = super::matching_files(root, &compiled.include, &selected, &target_roots)?;
        Ok(scan_files(root, &compiled, &files, sources))
    })
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
        defer_suppression: false,
    }
}

fn scan_files(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let mut findings: Vec<_> = files
        .par_iter()
        .flat_map(|path| scan::check_file(root, path, opts, sources))
        .collect();
    let present: BTreeSet<_> = files
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(root, path))
        .collect();
    for key in opts.allow.keys() {
        let path = key.split_once('#').map_or(key.as_str(), |(path, _)| path);
        if !present.contains(path) {
            findings.push(yaml::finding(
                path,
                1,
                format!("{path}: stale {RULE_ID} allow entry `{key}`"),
                Some(key),
            ));
        }
    }
    for key in &opts.duplicate_allow {
        let path = key.split_once('#').map_or(key.as_str(), |(path, _)| path);
        if !present.contains(path) {
            findings.push(yaml::finding(
                path,
                1,
                format!("{path}: duplicate {RULE_ID} allow entry `{key}`"),
                Some(key),
            ));
        }
    }
    findings
}

mod scan;
mod steps;
#[cfg(test)]
mod tests;
mod yaml;
