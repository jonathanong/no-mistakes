use super::masked_source::FileAllow;
use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod ast;
mod python;
mod scan;

pub const RULE_ID: &str = "no-raw-ephemeral-port";

pub(super) const DEFAULT_MESSAGE: &str =
    "raw ephemeral port 0 bind/listen can occupy a deterministic runner slice";

const DEFAULT_INCLUDE: &[&str] = &[
    "**/*.bash",
    "**/*.cjs",
    "**/*.cts",
    "**/*.js",
    "**/*.jsx",
    "**/*.mjs",
    "**/*.mts",
    "**/*.py",
    "**/*.sh",
    "**/*.ts",
    "**/*.tsx",
    "**/*.yaml",
    "**/*.yml",
    "**/*.zsh",
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) allow: Vec<String>,
    pub(crate) message: Option<String>,
}

pub(super) struct CompiledOptions {
    include: Vec<String>,
    allow: FileAllow,
    bind: regex::Regex,
    message: String,
}

super::borrowed_source_rule_entrypoints!();

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, all_files, sources, defer_suppression)
        .map(|output| output.findings)
}

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<super::analysis_policy::AnalysisDiagnostics> {
    let mut diagnostics = super::analysis_policy::AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let mut opts = compile_options(rule.try_rule_options()?)?;
        opts.allow.setup(root, config, rule, sources);
        let (files, target_roots) = super::selected_rule_files_with_roots_and_sources(
            root, config, rule, all_files, sources,
        )?;
        if files.is_empty() {
            if opts.allow.audits_missing_files() {
                if let Ok(matcher) = super::file_matching::FileMatcher::new(
                    root,
                    &target_roots,
                    opts.include.iter().map(String::as_str),
                ) {
                    diagnostics
                        .findings
                        .extend(opts.allow.finish_missing_selected(|file| {
                            matcher.is_match(&crate::codebase::ts_resolver::normalize_path(
                                &root.join(file),
                            ))
                        }));
                }
            }
            continue;
        }
        let matcher = super::file_matching::FileMatcher::new(
            root,
            &target_roots,
            opts.include.iter().map(String::as_str),
        )?;
        let files = matcher.select(&files);
        let mut findings = scan_files(root, &opts, &files, sources, defer_suppression);
        findings.extend(opts.allow.finish_selected(root, sources, |file| {
            matcher.is_match(&crate::codebase::ts_resolver::normalize_path(
                &root.join(file),
            ))
        }));
        diagnostics.append(super::file_diagnostics::application(
            root,
            sources,
            config,
            rule,
            findings,
            defer_suppression,
            super::file_diagnostics::InputUncertainty::Read,
        )?);
    }
    Ok(super::file_diagnostics::sorted(diagnostics))
}

fn compile_options(opts: Options) -> Result<CompiledOptions> {
    let include = if opts.include.is_empty() {
        DEFAULT_INCLUDE
            .iter()
            .map(|pattern| (*pattern).to_string())
            .collect()
    } else {
        opts.include
    };
    Ok(CompiledOptions {
        include,
        allow: FileAllow::new(&opts.allow, &format!("{RULE_ID} allow"))?,
        bind: regex::Regex::new(python::BIND_PATTERN).expect("bind pattern"),
        message: match opts.message.filter(|hint| !hint.is_empty()) {
            Some(hint) => format!("{DEFAULT_MESSAGE} {hint}"),
            None => DEFAULT_MESSAGE.to_string(),
        },
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

#[cfg(test)]
mod literal_recall_tests;
