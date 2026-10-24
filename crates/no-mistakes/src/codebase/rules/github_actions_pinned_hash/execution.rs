use super::*;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::config::v2::NoMistakesConfig;
use rayon::prelude::*;

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let outputs: Result<Vec<_>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let opts: Options = rule.try_rule_options()?;
            let selected = super::super::selected_rule_files(root, config, rule, files)?;
            let findings = scan(root, &opts, &selected, sources)?;
            super::super::file_diagnostics::application(
                root,
                sources,
                config,
                rule,
                findings,
                defer_suppression,
                super::super::file_diagnostics::InputUncertainty::ReadOrParse,
            )
        })
        .collect();
    let mut diagnostics = AnalysisDiagnostics::default();
    for output in outputs? {
        diagnostics.append(output);
    }
    Ok(super::super::file_diagnostics::sorted(diagnostics))
}

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, files, sources, defer_suppression)
        .map(|output| output.findings)
}

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let exclude_set = build_exclude_globset(&opts.exclude_paths);
    let (uses_re, sha_re, version_re) = build_patterns();
    let findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| {
            check_file_with_sources(
                path,
                root,
                &exclude_set,
                &uses_re,
                &sha_re,
                &version_re,
                sources,
            )
        })
        .collect();
    Ok(findings)
}
