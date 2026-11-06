use super::scan::check_source;
use super::scan::unresolved;
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
    use rules::file_diagnostics::{application, sorted, InputUncertainty};
    let mut output = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let _: Options = rule.try_rule_options()?;
        let selected = rules::selected_rule_files(root, config, rule, files)?;
        let findings = selected
            .par_iter()
            .flat_map(|path| check_file(root, path, sources, Unanalyzable::Report, true).findings)
            .collect();
        output.append(application(
            root,
            sources,
            config,
            rule,
            findings,
            defer_suppression,
            InputUncertainty::ReadOrParse,
        )?);
    }
    Ok(sorted(output))
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
        Ok(source) => match sources.parse_yaml_path(path) {
            Ok(value) => check_source(&rel, &source, &value, policy),
            Err(error) => {
                let mut diagnostics = AnalysisDiagnostics::default();
                diagnostics.unanalyzable(
                    policy,
                    unresolved(&rel, 1, "unanalyzable", &format!("invalid YAML ({error})")),
                );
                diagnostics
            }
        },
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
