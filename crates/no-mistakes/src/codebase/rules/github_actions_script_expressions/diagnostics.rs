use super::{analyze, compile_patterns, Options, RULE_ID};
use crate::codebase::rules::{self, analysis_policy::AnalysisDiagnostics, RuleFinding};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use std::path::{Path, PathBuf};

pub(super) fn run(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut diagnostics = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        let patterns = compile_patterns(&options)?;
        let files = rules::selected_rule_files(root, config, rule, files)?;
        let policy = config.effective_rule_analysis(rule)?.unanalyzable;
        let per_file: Vec<_> = files
            .par_iter()
            .map(|path| check_file(path, root, &patterns, sources, policy, defer_suppression))
            .collect();
        for result in per_file {
            diagnostics.append(result);
        }
    }
    if !defer_suppression {
        rules::suppress_rule_findings_with_sources(root, &mut diagnostics.findings, sources);
        rules::suppress_rule_findings_with_sources(root, &mut diagnostics.advisories, sources);
    }
    rules::sort_findings(&mut diagnostics.findings);
    rules::sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}

fn check_file(
    path: &Path,
    root: &Path,
    patterns: &[Regex],
    sources: &SourceStore,
    policy: Unanalyzable,
    defer_suppression: bool,
) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    let rel = relative_slash_path(root, path);
    if analyze::gha_kind(&rel).is_none() {
        return diagnostics;
    }
    match sources.read_path(path) {
        Ok(content) => diagnostics.findings = analyze::check_source(&rel, &content, patterns),
        Err(error) => {
            let finding = rules::github_actions_workflow_files::unreadable_finding(
                &error,
                defer_suppression,
                |line| {
                    RuleFinding {
                rule: RULE_ID.to_string(), file: rel.clone(), line,
                message: format!("{rel}: cannot read GitHub Actions source as UTF-8 ({error}); script expressions could not be checked. Restore a readable UTF-8 file so interpolated input cannot bypass script-injection checks."),
                import: None, target: Some("unanalyzable".to_string()),
            }
                },
            );
            if let Some(finding) = finding {
                diagnostics.unanalyzable(policy, finding);
            }
        }
    }
    diagnostics
}
