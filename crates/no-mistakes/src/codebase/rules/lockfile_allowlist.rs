use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "lockfile-allowlist";

const DEFAULT_ALLOWED: &[&str] = &["pnpm-lock.yaml"];
const DEFAULT_BANNED_BASENAMES: &[&str] = &[
    "package-lock.json",
    "npm-shrinkwrap.json",
    "yarn.lock",
    "bun.lock",
    "bun.lockb",
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) allowed: Vec<String>,
    pub(crate) banned_basenames: Vec<String>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let files: Vec<PathBuf> = target_roots
                .iter()
                .flat_map(|r| discover_files(r, skip))
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, &opts, &files)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| target_roots.iter().any(|r| p.starts_with(r)))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, &opts, &files)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn build_glob_set(patterns: &[&str]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pat in patterns {
        builder.add(Glob::new(pat)?);
    }
    Ok(builder.build()?)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let banned: HashSet<&str> = if opts.banned_basenames.is_empty() {
        DEFAULT_BANNED_BASENAMES.iter().copied().collect()
    } else {
        opts.banned_basenames.iter().map(String::as_str).collect()
    };

    let allowed_patterns: Vec<&str> = if opts.allowed.is_empty() {
        DEFAULT_ALLOWED.to_vec()
    } else {
        opts.allowed.iter().map(String::as_str).collect()
    };

    let glob_set = build_glob_set(&allowed_patterns)?;

    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| {
            let basename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if !banned.contains(basename) {
                return Vec::new();
            }
            let rel = relative_slash_path(root, path);
            if glob_set.is_match(&rel) {
                return Vec::new();
            }
            vec![RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: 1,
                message: format!(
                    "{rel}: lockfile is not allowed — remove it or add to the allowlist"
                ),
                import: None,
                target: None,
            }]
        })
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(findings)
}

#[cfg(test)]
mod tests;
