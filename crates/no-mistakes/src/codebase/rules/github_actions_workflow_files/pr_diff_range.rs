use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::{self, RuleFinding};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

mod checkout;
mod diff;
mod env_read;
mod exprs;
mod options;
mod pairs;
mod pairs_locate;
mod revisions;
mod runs;
mod shell_guard;
mod step_judge;
mod verdicts;
mod verdicts_helpers;
mod yaml_shapes;

pub(crate) use options::collect_config_errors;

pub const RULE_ID: &str = "github-actions-pr-diff-range";

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    rules::check_with_borrowed_source_store(root, config, all_files, |r, c, f, s| {
        check_with_files_and_sources(r, c, f, s, false)
    })
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, all_files, sources, defer_suppression)
        .map(|output| output.findings)
}

fn resolve_unanalyzable(
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
) -> Result<Unanalyzable> {
    let explicit = rule.options.get("unanalyzable").is_some() || config.unanalyzable.is_some();
    if explicit {
        Ok(config.effective_rule_analysis(rule)?.unanalyzable)
    } else {
        Ok(Unanalyzable::Advise)
    }
}

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let raw_opts: options::Options = rule.try_rule_options()?;
        let compiled = options::compile(&raw_opts)?;
        let unanalyzable = resolve_unanalyzable(config, rule)?;
        let files = rules::selected_rule_files(root, config, rule, all_files)?;
        let chunks: Vec<AnalysisDiagnostics> = files
            .par_iter()
            .map(|path| scan_file(root, path, &compiled, sources, defer_suppression, unanalyzable))
            .collect();
        for chunk in chunks {
            output.append(chunk);
        }
    }
    rules::sort_findings(&mut output.findings);
    rules::sort_findings(&mut output.advisories);
    Ok(output)
}

fn scan_file(
    root: &Path,
    path: &Path,
    compiled: &options::CompiledOptions,
    sources: &SourceStore,
    defer_suppression: bool,
    unanalyzable: Unanalyzable,
) -> AnalysisDiagnostics {
    let mut output = AnalysisDiagnostics::default();
    let rel = relative_slash_path(root, path);
    if super::gha_kind(&rel).is_none() {
        return output;
    }
    let Some(content) = rules::read_language_source(sources, path) else {
        return output;
    };
    if !defer_suppression && crate::codebase::ts_source::has_disable_file_comment(&content, RULE_ID)
    {
        return output;
    }
    let (findings, undecidable) = runs::check_source_with_options(&rel, &content, compiled);
    output.findings = findings;
    for finding in undecidable {
        output.unanalyzable(unanalyzable, finding);
    }
    if !defer_suppression {
        rules::suppress_rule_findings_with_source(&mut output.findings, &content);
        rules::suppress_rule_findings_with_source(&mut output.advisories, &content);
    }
    output
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_consistent_pairs;
