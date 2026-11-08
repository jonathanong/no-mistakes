use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
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
    let files =
        crate::codebase::ts_source::discover_files(root, &config.filesystem.skip_directories);
    let sources = super::source_access::complete_source_store_for_files(&files);
    check_with_files_and_sources(root, config, &files, &sources)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let opts: Options = rule.try_rule_options()?;
            let files =
                super::selected_rule_files_with_sources(root, config, rule, all_files, sources)?;
            let audit = super::allow_audit::AllowAudit::new_files(
                root,
                config,
                rule,
                Some(sources),
                "allowed",
                opts.allowed.len(),
            );
            let scope = super::allow_audit::CheckedScope::new(root, Some(sources), &files);
            let matchers = opts
                .allowed
                .iter()
                .map(|pattern| Glob::new(pattern).map(|glob| glob.compile_matcher()))
                .collect::<Result<Vec<_>, _>>()?;
            let mut findings = scan(root, &opts, &files, opts.allowed.is_empty())?;
            findings.retain(|finding| {
                !audit.usage.suppress(
                    matchers
                        .iter()
                        .map(|matcher| matcher.is_match(&finding.file)),
                )
            });
            findings.extend(audit.finish_files(
                |index| scope.glob(&opts.allowed[index]),
                |index| opts.allowed[index].clone(),
                |index| opts.allowed[index].clone(),
            ));
            Ok(findings)
        })
        .collect();
    let mut findings = all?.into_iter().flatten().collect();
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

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    apply_allow: bool,
) -> Result<Vec<RuleFinding>> {
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
            if apply_allow && glob_set.is_match(&rel) {
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
