use super::{analysis_policy::AnalysisDiagnostics, RuleFinding};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::Result;
use std::path::Path;

pub(crate) enum InputUncertainty {
    Read,
    ReadOrParse,
}

pub(crate) fn application(
    root: &Path,
    sources: &SourceStore,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    mut findings: Vec<RuleFinding>,
    defer_suppression: bool,
    uncertainty: InputUncertainty,
) -> Result<AnalysisDiagnostics> {
    if !defer_suppression {
        super::suppress_rule_findings_with_sources(root, &mut findings, sources);
        findings.retain(|finding| {
            if finding.target.as_deref() != Some("unanalyzable") {
                return true;
            }
            let path = root.join(&finding.file);
            if let Err(error) = sources.read_path(&path) {
                // SourceStore retains invalid bytes in its cached failure; no filesystem retry.
                let mut candidate = vec![finding.clone()];
                super::source_access::suppress_invalid_utf8(&mut candidate, &error);
                return !candidate.is_empty();
            }
            true
        });
    }
    route_application(config, rule, findings, |finding| match uncertainty {
        InputUncertainty::Read => sources.read_path(&root.join(&finding.file)).is_err(),
        InputUncertainty::ReadOrParse => true,
    })
}

pub(crate) fn route_application(
    config: &NoMistakesConfig,
    rule: &RuleDef,
    findings: Vec<RuleFinding>,
    uncertain: impl Fn(&RuleFinding) -> bool,
) -> Result<AnalysisDiagnostics> {
    let policy = config.effective_rule_analysis(rule)?.unanalyzable;
    let mut diagnostics = AnalysisDiagnostics::default();
    for mut finding in findings {
        if let Some(message) = &rule.message {
            finding.message = message.clone();
        }
        if finding.target.as_deref() == Some("unanalyzable") && uncertain(&finding) {
            diagnostics.unanalyzable(policy, finding);
        } else {
            diagnostics.findings.push(finding);
        }
    }
    Ok(diagnostics)
}

pub(crate) fn sorted(mut diagnostics: AnalysisDiagnostics) -> AnalysisDiagnostics {
    super::sort_findings(&mut diagnostics.findings);
    super::sort_findings(&mut diagnostics.advisories);
    diagnostics
}

pub fn handles(rule: &str) -> bool {
    matches!(
        rule,
        super::RUST_MAX_LINES_PER_FILE
            | super::RUST_NO_INLINE_ALLOWS
            | super::RUST_NO_INLINE_TESTS
            | super::CSHARP_MAX_LINES_PER_FILE
            | super::NO_EMPTY_OR_COMMENTS_ONLY_FILES
            | super::NO_RAW_EPHEMERAL_PORT
            | super::STRUCTURED_CONFIG_POLICY
            | super::github_actions_pinned_hash::RULE_ID
    )
}

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[std::path::PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    type Engine = fn(
        &Path,
        &NoMistakesConfig,
        &[std::path::PathBuf],
        &SourceStore,
        bool,
    ) -> Result<AnalysisDiagnostics>;
    let engines: &[Engine] = &[
        super::rust_rules_combined::diagnostics_with_files_and_sources,
        super::csharp_max_lines_per_file::diagnostics_with_files_and_sources,
        super::no_empty_or_comments_only_files::diagnostics_with_files_and_sources,
        super::no_raw_ephemeral_port::diagnostics_with_files_and_sources,
        super::structured_config_policy::diagnostics_with_files_and_sources,
        super::github_actions_pinned_hash::diagnostics_with_files_and_sources,
    ];
    use rayon::prelude::*;
    let outputs: Result<Vec<_>> = engines
        .par_iter()
        .map(|engine| engine(root, config, files, sources, defer_suppression))
        .collect();
    let mut diagnostics = AnalysisDiagnostics::default();
    for output in outputs? {
        diagnostics.append(output);
    }
    Ok(sorted(diagnostics))
}

#[cfg(test)]
mod tests;

mod applications;
pub(crate) use applications::{read_applications, FileRuleInputs};
