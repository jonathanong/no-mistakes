use super::emit::finding;
use super::options::{self, CompiledOptions, Options};
use super::workflow;
use super::RULE_ID;
use crate::codebase::rules::{
    self,
    analysis_policy::AnalysisDiagnostics,
    github_actions_workflow_files::{gha_kind, unreadable_finding},
    RuleFinding,
};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    rules::check_sourced_standalone(root, config, check_with_files)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    rules::check_with_borrowed_source_store(root, config, all_files, |r, c, f, s| {
        check_with_files_sources_and_deferred_suppression(r, c, f, s, false)
    })
}

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, files, sources, defer_suppression)
        .map(|output| output.findings)
}

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    run(root, config, files, sources, defer_suppression)
}

pub(super) struct RunCtx<'a> {
    pub(super) root: &'a Path,
    pub(super) sources: &'a SourceStore,
    pub(super) compiled: &'a CompiledOptions,
    pub(super) unanalyzable: Unanalyzable,
    pub(super) rule_message: Option<&'a str>,
    pub(super) defer_suppression: bool,
}

pub(super) fn run(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let raw: Options = rule.try_rule_options()?;
        let compiled = options::compile(raw)?;
        let (selected, roots) = rules::selected_rule_files_with_roots(root, config, rule, files)?;
        let selected = if compiled.include.is_empty() {
            selected
        } else {
            rules::matching_files(root, &compiled.include, &selected, &roots)?
        };
        let unanalyzable = resolve_unanalyzable(config, rule)?;
        if compiled.script_dirs.is_empty() {
            continue;
        }
        let ctx = RunCtx {
            root,
            sources,
            compiled: &compiled,
            unanalyzable,
            rule_message: rule.message.as_deref(),
            defer_suppression,
        };
        let chunks: Vec<_> = selected
            .par_iter()
            .filter_map(|path| {
                let rel = relative_slash_path(root, path);
                (gha_kind(&rel) == Some(false)).then(|| check_path(&ctx, path, &rel))
            })
            .collect();
        for chunk in chunks {
            output.append(chunk);
        }
    }
    if !defer_suppression {
        rules::suppress_rule_findings_with_sources(root, &mut output.findings, sources);
        rules::suppress_rule_findings_with_sources(root, &mut output.advisories, sources);
    }
    rules::sort_findings(&mut output.findings);
    rules::sort_findings(&mut output.advisories);
    Ok(output)
}

fn check_path(ctx: &RunCtx<'_>, path: &Path, rel: &str) -> AnalysisDiagnostics {
    match ctx.sources.read_path(path) {
        Ok(source) => workflow::check_source(ctx, path, rel, &source),
        Err(error) => {
            let mut diag = AnalysisDiagnostics::default();
            let item = unreadable_finding(&error, ctx.defer_suppression, |line| {
                finding(
                    rel,
                    line,
                    format!(
                        "{rel}:{line}: cannot read workflow as UTF-8 ({error}); fix the file encoding so workflow path filters can be checked."
                    ),
                    "unanalyzable",
                )
            });
            if let Some(item) = item {
                diag.unanalyzable(ctx.unanalyzable, item);
            }
            diag
        }
    }
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
