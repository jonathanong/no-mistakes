use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod lines;
mod message;
mod policy;
mod scalar;
mod scan;

pub const RULE_ID: &str = "github-actions-action-input-policy";

const DEFAULT_INCLUDE: &[&str] = &[
    ".github/workflows/**/*.yml",
    ".github/workflows/**/*.yaml",
    ".github/actions/**/action.yml",
    ".github/actions/**/action.yaml",
];

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) actions: Vec<ActionSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionSpec {
    pub(crate) uses: String,
    #[serde(default)]
    pub(crate) require: BTreeMap<String, Value>,
    #[serde(default)]
    pub(crate) ban: Vec<String>,
    #[serde(default, rename = "allowedInputs")]
    pub(crate) allowed_inputs: Option<Vec<String>>,
    #[serde(default, rename = "requiredPresent")]
    pub(crate) required_present: Vec<String>,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources, false)
}

pub(crate) fn collect_config_errors(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    errors: &mut Vec<String>,
) {
    for (index, rule) in super::validate_options::indexed_rule_applications(config, RULE_ID) {
        let mut app_errors = std::collections::BTreeSet::new();
        if let Err(error) = super::path_filter::RulePathFilter::new(root, config, rule) {
            app_errors.insert(format!("{error:#}"));
        }
        match rule.try_rule_options::<Options>() {
            Ok(opts) => {
                for pattern in &opts.include {
                    if let Err(error) = globset::Glob::new(pattern.trim_start_matches("./")) {
                        app_errors.insert(super::validate_options::format_invalid_option(
                            RULE_ID,
                            "include",
                            pattern,
                            format_args!("a valid glob ({error})"),
                        ));
                    }
                }
                policy::collect_errors(opts, &mut app_errors);
            }
            Err(error) => {
                app_errors.insert(format!("{error:#}"));
            }
        }
        super::validate_options::push_application_errors(
            root,
            config,
            index,
            config_path,
            app_errors,
            errors,
        );
    }
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let mut errors = Vec::new();
    collect_config_errors(root, None, config, &mut errors);
    super::validate_options::finish(errors)?;
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let mut opts = policy::compile(rule.try_rule_options()?)?;
        if opts.actions.is_empty() {
            continue;
        }
        opts.target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| {
                super::file_allowed_by_roots_and_skip(root, &skip, path, &opts.target_roots)
            })
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        if files.is_empty() {
            continue;
        }
        let include = if opts.include.is_empty() {
            DEFAULT_INCLUDE
                .iter()
                .map(|pattern| (*pattern).to_string())
                .collect()
        } else {
            opts.include.clone()
        };
        let files = super::matching_files(root, &include, &files, &opts.target_roots)?;
        findings.extend(scan_files(root, &opts, &files, sources, defer_suppression));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan_files(
    root: &Path,
    opts: &policy::Compiled,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    files
        .par_iter()
        .flat_map(|path| scan::check_file(root, path, opts, sources, defer_suppression))
        .collect()
}

#[cfg(test)]
mod tests;
