use super::github_actions_workflow_files::gha_kind;
use super::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

mod broad_hash;
mod expand;
mod forbidden;
mod judge;
mod local_action;
mod options;
mod output;
mod output_syntax;
mod scan;
mod shell_cmd;
mod text;

pub const RULE_ID: &str = "github-actions-cache-key-policy";

super::borrowed_source_rule_entrypoints!();

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    super::check_sourced_standalone(&root, config, check_with_files)
}

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let mut findings = super::run_rule_applications(config, RULE_ID, |rule| {
        let opts: options::Options = rule.try_rule_options()?;
        options::validate(&opts)?;
        let rule_message = rule
            .message
            .as_deref()
            .map(str::trim)
            .filter(|message| !message.is_empty());
        let files = super::selected_rule_files(root, config, rule, files)?;
        Ok(files
            .par_iter()
            .flat_map(|path| {
                check_path(root, path, sources, &opts, rule_message, defer_suppression)
            })
            .collect())
    })?;
    if !defer_suppression {
        super::suppress_rule_findings_with_sources(root, &mut findings, sources);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn check_path(
    root: &Path,
    path: &Path,
    sources: &SourceStore,
    options: &options::Options,
    rule_message: Option<&str>,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    let Some(action) = gha_kind(&rel) else {
        return Vec::new();
    };
    match sources.read_path(path) {
        Ok(source) => evaluate(&rel, &source, action, options, rule_message, root, sources),
        Err(error) => super::github_actions_workflow_files::unreadable_finding(
            &error,
            defer_suppression,
            |line| {
                hit(
                    &rel,
                    line,
                    format!(
                        "Cannot read GitHub Actions YAML as UTF-8 ({error}); restore a readable file so cache keys can be checked."
                    ),
                )
            },
        )
        .into_iter()
        .collect(),
    }
}

fn evaluate(
    rel: &str,
    source: &str,
    action: bool,
    options: &options::Options,
    rule_message: Option<&str>,
    root: &Path,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let configured: Vec<String> = options
        .caches
        .iter()
        .map(|cache| text::norm_path(&cache.path))
        .chain(
            options
                .forbidden_paths
                .iter()
                .map(|path| text::norm_path(path)),
        )
        .collect();
    match scan::analyze(source, action, &configured) {
        scan::Document::Ignored => Vec::new(),
        scan::Document::Invalid { line, detail } => vec![hit(
            rel,
            line,
            format!("Invalid YAML ({detail}); fix the YAML so cache keys can be checked."),
        )],
        scan::Document::Unaligned => vec![hit(
            rel,
            1,
            "Cache steps could not be aligned to block-style source locations. Rewrite anchors, aliases, and unusual flow as ordinary block-style steps so cache keys can be checked.".into(),
        )],
        scan::Document::Jobs(jobs) => {
            judge::judge_with_message(rel, &jobs, options, rule_message, &|uses| {
                read_local_action(root, sources, uses)
            })
        }
    }
}

fn read_local_action(root: &Path, sources: &SourceStore, uses: &str) -> Option<String> {
    let rel = local_action::local_action_dir(uses)?;
    for name in ["action.yml", "action.yaml"] {
        let path = root.join(rel).join(name);
        if let Ok(text) = sources.read_path(&path) {
            return Some(text.to_string());
        }
    }
    None
}

fn hit(file: &str, line: usize, message: String) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message,
        import: None,
        target: None,
    }
}

#[cfg(test)]
mod tests;
