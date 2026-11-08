use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "github-actions-test-timeout-literals";

const DEFAULT_INCLUDE: &[&str] = &[
    ".github/workflows/**/*.test.mts",
    ".github/workflows/**/*.test.ts",
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) allow: Vec<AllowEntry>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct AllowEntry {
    pub(crate) file: String,
    pub(crate) text: String,
    pub(crate) reason: String,
}

struct CompiledOptions {
    allow: BTreeMap<String, String>,
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
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let raw: Options = rule.try_rule_options()?;
        validate_options(&raw)?;
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
                let file = entry.file.trim();
                let selected = matcher.is_match(&crate::codebase::ts_resolver::normalize_path(
                    &root.join(file),
                ));
                (
                    if selected {
                        file.to_string()
                    } else {
                        String::new()
                    },
                    format!("{}#{}", file, entry.text.trim()),
                )
            }),
        );
        let opts = compile_options(raw)?;
        findings.extend(missing);
        let files = matcher.select(&selected);
        findings.extend(scan_files(root, &opts, &files, sources, defer_suppression));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn validate_options(opts: &Options) -> Result<()> {
    for (index, entry) in opts.allow.iter().enumerate() {
        anyhow::ensure!(
            !entry.file.trim().is_empty(),
            "{RULE_ID}: allow[{index}].file is required and must not be empty"
        );
    }
    Ok(())
}

fn compile_options(opts: Options) -> Result<CompiledOptions> {
    validate_options(&opts)?;
    Ok(CompiledOptions {
        allow: opts
            .allow
            .into_iter()
            .map(|entry| {
                (
                    format!("{}#{}", entry.file.trim(), entry.text.trim()),
                    entry.reason,
                )
            })
            .collect(),
    })
}

fn scan_files(
    root: &Path,
    opts: &CompiledOptions,
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
