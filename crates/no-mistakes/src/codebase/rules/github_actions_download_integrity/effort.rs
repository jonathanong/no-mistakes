use super::*;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::{Effort, NoMistakesConfig};

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    super::super::check_sourced_standalone(root, config, check_with_files)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    super::super::check_with_borrowed_source_store(
        root,
        config,
        files,
        check_with_files_and_sources,
    )
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    // The compatibility file-rule boundary leaves suppression to its caller.
    Ok(diagnostics_with_files_and_sources(root, config, files, sources, true)?.findings)
}

/// Evaluate every application once, retaining its independent analysis policy.
#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut diagnostics = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        let files = super::super::selected_rule_files(root, config, rule, files)?;
        let policy = config.effective_rule_analysis(rule)?;
        if policy.effort != Effort::Low {
            diagnostics
                .findings
                .extend(scan(root, &options, &files, sources)?);
            continue;
        }
        let (hosts, helpers) = compile(&options)?;
        let chunks: Vec<_> = files
            .par_iter()
            .map(|path| {
                let rel = relative_slash_path(root, path);
                if gha_kind(&rel).is_none() {
                    return AnalysisDiagnostics::default();
                }
                let Some(content) = super::super::read_source(sources, path) else {
                    return AnalysisDiagnostics::default();
                };
                runs::check_source_low(&rel, &content, &hosts, &helpers, policy.unanalyzable)
            })
            .collect();
        for chunk in chunks {
            diagnostics.append(chunk);
        }
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
