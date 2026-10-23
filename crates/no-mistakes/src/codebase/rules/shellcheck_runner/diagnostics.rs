use super::*;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use rayon::prelude::*;

/// Request boundary consumes findings and advisories from one ShellCheck execution.
#[doc(hidden)]
pub fn diagnostics_with_files_sources_and_snapshot(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    snapshot: &VisiblePathSnapshot,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let all: Result<Vec<_>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let opts: Options = rule.try_rule_options()?;
            let policy = config.effective_rule_analysis(rule)?.unanalyzable;
            let target_roots = super::super::target_roots(root, config, rule);
            let skip = super::super::skip_dir_set(config);
            let files: Vec<_> = all_files
                .iter()
                .filter(|path| {
                    super::super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
                })
                .cloned()
                .collect();
            let files = super::super::path_filter::filter_rule_files(root, config, rule, &files)?;
            let filter = super::super::path_filter::RulePathFilter::new(root, config, rule)?;
            let findings = selection::scan_with_sources(
                root,
                &opts,
                &files,
                &target_roots,
                &filter,
                sources,
                snapshot,
            )?;
            Ok(route_findings(policy, rule.message.as_deref(), findings))
        })
        .collect();
    let mut diagnostics = AnalysisDiagnostics::default();
    for next in all? {
        diagnostics.append(next);
    }
    if !defer_suppression {
        super::super::suppress_rule_findings_with_sources(root, &mut diagnostics.findings, sources);
        super::super::suppress_rule_findings_with_sources(
            root,
            &mut diagnostics.advisories,
            sources,
        );
    }
    super::super::sort_findings(&mut diagnostics.findings);
    super::super::sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}

pub(super) fn route_findings(
    policy: crate::config::v2::Unanalyzable,
    message: Option<&str>,
    findings: Vec<RuleFinding>,
) -> AnalysisDiagnostics {
    let mut diagnostics = AnalysisDiagnostics::default();
    for mut finding in findings {
        if let Some(message) = message {
            finding.message = message.to_string();
        }
        if finding.target.as_deref() == Some("unanalyzable") {
            diagnostics.unanalyzable(policy, finding);
        } else {
            diagnostics.findings.push(finding);
        }
    }
    diagnostics
}
