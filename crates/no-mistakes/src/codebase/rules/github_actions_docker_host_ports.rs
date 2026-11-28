use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use crate::codebase::rules::file_diagnostics::{self, InputUncertainty};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use no_mistakes_rule_kernel::finding::RuleFinding;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub(super) use super::github_actions_workflow_files::gha_kind;

mod flags;
mod mapping;
mod ports;
mod publish;
mod runs;
mod shell;

pub const RULE_ID: &str = "github-actions-docker-host-ports";

/// The rule has no options. A non-empty options map is rejected.
#[derive(Debug, Default, Deserialize)]
pub(in crate::codebase::rules) struct Options;

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
        let (mut findings, input_failures) = scan(root, &options, &selected, sources);
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
    _opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> (Vec<RuleFinding>, Vec<RuleFinding>) {
    super::github_actions_workflow_files::scan_yaml_files(
        root,
        files,
        sources,
        RULE_ID,
        &gha_kind,
        &|rel, content, value, action| runs::check_parsed(rel, content, value, action),
    )
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
