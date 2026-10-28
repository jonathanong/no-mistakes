use super::findings::unresolved;
use super::scan::check_source;
use super::{Options, RULE_ID};
use crate::codebase::rules::{self, analysis_policy::AnalysisDiagnostics};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use rayon::prelude::*;
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
        let _: Options = rule.try_rule_options()?;
        let selected = rules::selected_rule_files(root, config, rule, files)?;
        let policy = config.effective_rule_analysis(rule)?.unanalyzable;
        let per_file: Vec<_> = selected
            .par_iter()
            .map(|path| check_file(root, path, sources, policy, defer_suppression))
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
    root: &Path,
    path: &Path,
    sources: &SourceStore,
    policy: Unanalyzable,
    defer_suppression: bool,
) -> AnalysisDiagnostics {
    let rel = relative_slash_path(root, path);
    if rules::github_actions_workflow_files::gha_kind(&rel) != Some(false) {
        return AnalysisDiagnostics::default();
    }
    match sources.read_path(path) {
        Ok(source) => check_source(&rel, &source, policy),
        Err(error) => {
            let mut diagnostics = AnalysisDiagnostics::default();
            let finding = rules::github_actions_workflow_files::unreadable_finding(
                &error,
                defer_suppression,
                |line| {
                    unresolved(
                        &rel,
                        line,
                        "unanalyzable",
                        &format!("cannot read workflow source as UTF-8 ({error})"),
                    )
                },
            );
            if let Some(finding) = finding {
                diagnostics.unanalyzable(policy, finding);
            }
            diagnostics
        }
    }
}
