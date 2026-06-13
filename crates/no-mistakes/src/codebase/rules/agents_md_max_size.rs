use super::RuleFinding;
use crate::codebase::ts_source::discover_with_basenames;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "agents-md-max-size";

mod agents_md_max_size_budget;
use agents_md_max_size_budget::{scan, scan_advisories};

const DEFAULT_MAX_LINES: usize = 200;
const DEFAULT_MAX_CHARS: usize = 12_000;
const DEFAULT_FILENAMES: &[&str] = &["AGENTS.md", "CLAUDE.md"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) max_lines: Option<usize>,
    pub(crate) max_chars: Option<usize>,
    pub(crate) advisory_chars_remaining: Option<usize>,
    pub(crate) filenames: Option<Vec<String>>,
    pub(crate) roots: Option<Vec<PathBuf>>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let filenames = filenames_from_opts(&opts);
        let target_roots = super::target_roots(root, config, rule);
        let roots = roots_from_opts(&opts, root, &target_roots);
        let files: Vec<PathBuf> = roots
            .iter()
            .flat_map(|r| discover_with_basenames(r, skip, &filenames))
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub fn advisories_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut advisories = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        if opts.advisory_chars_remaining.is_none() {
            continue;
        }
        let filenames = filenames_from_opts(&opts);
        let target_roots = super::target_roots(root, config, rule);
        let roots = roots_from_opts(&opts, root, &target_roots);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|p| {
                super::file_allowed_by_roots_and_skip(root, &skip, p, &roots)
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| filenames.contains(&n))
            })
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        advisories.extend(scan_advisories(root, &opts, &files)?);
    }
    super::sort_findings(&mut advisories);
    super::suppress_rule_findings(root, &mut advisories);
    Ok(advisories)
}

/// Check using a pre-discovered file list to avoid a second filesystem walk.
pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = rule.rule_options();
        let filenames = filenames_from_opts(&opts);
        let target_roots = super::target_roots(root, config, rule);
        let roots = roots_from_opts(&opts, root, &target_roots);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|p| {
                super::file_allowed_by_roots_and_skip(root, &skip, p, &roots)
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| filenames.contains(&n))
            })
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn filenames_from_opts(opts: &Options) -> Vec<&str> {
    opts.filenames
        .as_deref()
        .map(|v| v.iter().map(String::as_str).collect())
        .unwrap_or_else(|| DEFAULT_FILENAMES.to_vec())
}

fn roots_from_opts(opts: &Options, root: &Path, target_roots: &[PathBuf]) -> Vec<PathBuf> {
    opts.roots
        .as_deref()
        .map(|rs| {
            rs.iter()
                .map(|r| {
                    if r.is_absolute() {
                        r.clone()
                    } else {
                        root.join(r)
                    }
                })
                .collect()
        })
        .unwrap_or_else(|| target_roots.to_vec())
}

#[cfg(test)]
mod tests;
