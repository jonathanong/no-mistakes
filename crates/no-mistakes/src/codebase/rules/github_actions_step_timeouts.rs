use super::analysis_policy::AnalysisDiagnostics;
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use no_mistakes_rule_kernel::finding::RuleFinding;
use options::{compile_options, CompiledOptions};
pub(in crate::codebase::rules) use options::Options;
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "github-actions-step-timeouts";
const DEFAULT_INCLUDE: &[&str] = &[".github/workflows/**/*.yml", ".github/workflows/**/*.yaml"];
const GITHUB_MAX_MINUTES: u64 = 360;

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
    diagnostics_with_files_and_sources(root, config, all_files, sources, defer_suppression)
        .map(|output| output.findings)
}

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let raw: Options = rule.try_rule_options()?;
        let (selected, roots) = super::selected_rule_files_with_roots_and_sources(
            root, config, rule, all_files, sources,
        )?;
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
                let file = selector_file(&entry.step)
                    .filter(|file| {
                        matcher.is_match(&crate::codebase::ts_resolver::normalize_path(
                            &root.join(file),
                        ))
                    })
                    .unwrap_or_default()
                    .to_string();
                (file, entry.step.clone())
            }),
        );
        let unanalyzable = resolve_unanalyzable(config, rule)?;
        let compiled = compile_options(raw, unanalyzable, defer_suppression)?;
        let files = matcher.select(&selected);
        let mut chunk = scan_files(root, &compiled, &files, sources);
        chunk.findings.extend(missing);
        output.append(chunk);
    }
    super::sort_findings(&mut output.findings);
    super::sort_findings(&mut output.advisories);
    Ok(output)
}

fn scan_files(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> AnalysisDiagnostics {
    let chunks: Vec<_> = files
        .par_iter()
        .map(|path| scan::check_file(root, path, opts, sources))
        .collect();
    let mut output = AnalysisDiagnostics::default();
    for chunk in chunks {
        output.append(chunk);
    }
    let present: BTreeSet<_> = files
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(root, path))
        .collect();
    for key in opts.allow.keys() {
        if selector_file(key).is_none() {
            output.findings.push(yaml::finding(
                key,
                1,
                format!("{key}: malformed {RULE_ID} allow selector; expected file#job#step"),
                Some(key),
            ));
        }
    }
    for key in &opts.duplicate_allow {
        let Some(path) = selector_file(key) else {
            continue;
        };
        if !present.contains(path) {
            output.findings.push(yaml::finding(
                path,
                1,
                format!("{path}: duplicate {RULE_ID} allow entry `{key}`"),
                Some(key),
            ));
        }
    }
    output
}

fn resolve_unanalyzable(
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
) -> Result<Unanalyzable> {
    if rule.options.get("unanalyzable").is_some() || config.unanalyzable.is_some() {
        Ok(config.effective_rule_analysis(rule)?.unanalyzable)
    } else {
        Ok(Unanalyzable::Advise)
    }
}

fn selector_file(key: &str) -> Option<&str> {
    key.rsplit_once('#')
        .filter(|(_, step)| !step.is_empty())
        .and_then(|(file_and_job, _)| file_and_job.rsplit_once('#'))
        .filter(|(file, job)| {
            !job.is_empty() && (file.ends_with(".yml") || file.ends_with(".yaml"))
        })
        .map(|(file, _)| file)
}

mod dynamic;
mod options;
mod scan;
mod steps;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
mod yaml;

mod validation;
