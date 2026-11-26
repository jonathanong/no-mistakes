use crate::codebase::rules::self;
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::file_diagnostics::{self, InputUncertainty};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod loops;
mod runs;

pub const RULE_ID: &str = "github-actions-no-poll-wait-job";

/// `pollCommands` is empty by default, so the rule reports nothing until a
/// configuration names the commands that poll another job or workflow.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(in crate::codebase::rules) struct Options {
    poll_commands: Vec<String>,
    ignore_for_loops: bool,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    rules::check_with_borrowed_source_store(root, config, all_files, check_with_files_and_sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, all_files, sources, true)
        .map(|output| output.findings)
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
        let opts: Options = rule.try_rule_options()?;
        let files = rules::selected_rule_files(root, config, rule, all_files)?;
        let polls = compile(&opts)?;
        if polls.is_empty() {
            continue;
        }
        let (mut findings, failures) = super::scan_yaml_files(
            root,
            &files,
            sources,
            RULE_ID,
            &super::gha_kind,
            &|rel, source, value, action| {
                runs::check_parsed(rel, source, value, action, &polls, opts.ignore_for_loops)
            },
        );
        if !defer_suppression {
            rules::suppress_rule_findings_with_sources(root, &mut findings, sources);
        }
        output.findings.extend(findings);
        output.append(file_diagnostics::application(
            root,
            sources,
            config,
            rule,
            failures,
            defer_suppression,
            InputUncertainty::ReadOrParse,
        )?);
    }
    Ok(file_diagnostics::sorted(output))
}

pub(in crate::codebase::rules) fn compile(opts: &Options) -> Result<Vec<Vec<String>>> {
    opts.poll_commands
        .iter()
        .map(|command| {
            let words: Vec<String> = command.split_whitespace().map(str::to_string).collect();
            if words.is_empty() {
                anyhow::bail!("{RULE_ID} pollCommands entries must name a command");
            }
            Ok(words)
        })
        .collect()
}

#[cfg(test)]
mod tests;
