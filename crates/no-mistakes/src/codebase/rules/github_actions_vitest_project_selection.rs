use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::file_diagnostics::{self, InputUncertainty};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use no_mistakes_rule_kernel::finding::RuleFinding;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod command;
mod judge;
pub(in crate::codebase::rules) mod options;
mod runs;

pub const RULE_ID: &str = "github-actions-vitest-project-selection";

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Options {
    pub(crate) projects: Vec<ProjectOption>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ProjectOption {
    pub(crate) project: String,
    pub(crate) workflows: Vec<String>,
    pub(crate) name_filters: Option<String>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    super::check_sourced_standalone(root, config, check_with_files)
}

super::diagnostic_source_rule_entrypoints!();

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut diagnostics = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        let selected = super::selected_rule_files(root, config, rule, files)?;
        let compiled = options::compile(&options)?;
        let (mut findings, input_failures) = scan(root, &compiled, &selected, sources);
        if !defer_suppression {
            super::suppress_rule_findings_with_sources(root, &mut findings, sources);
        }
        diagnostics.findings.extend(findings);
        diagnostics.append(file_diagnostics::application(
            root,
            sources,
            config,
            rule,
            input_failures,
            defer_suppression,
            InputUncertainty::ReadOrParse,
        )?);
    }
    Ok(file_diagnostics::sorted(diagnostics))
}

fn scan(
    root: &Path,
    compiled: &options::Compiled,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> (Vec<RuleFinding>, Vec<RuleFinding>) {
    if compiled.projects.is_empty() {
        return (Vec::new(), Vec::new());
    }
    super::github_actions_workflow_files::scan_yaml_files(
        root,
        files,
        sources,
        RULE_ID,
        &|rel| {
            let action = super::github_actions_workflow_files::gha_kind(rel)?;
            compiled
                .projects
                .iter()
                .any(|policy| policy.workflows.is_match(rel))
                .then_some(action)
        },
        &|rel, content, value, action| runs::check_parsed(rel, content, value, action, compiled),
    )
}

#[cfg(test)]
mod tests;
