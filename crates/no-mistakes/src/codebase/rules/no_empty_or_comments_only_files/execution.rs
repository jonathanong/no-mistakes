use super::*;
use crate::codebase::ts_source::discover_files;

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let normalized_root = crate::codebase::ts_resolver::normalize_path(root);
    let root = normalized_root.as_path();
    let roots: std::collections::BTreeSet<_> = config
        .rule_applications(RULE_ID)
        .into_iter()
        .flat_map(|rule| super::super::target_roots(root, config, rule))
        .collect();
    let mut files: Vec<_> = roots
        .into_iter()
        .flat_map(|root| discover_files(&root, &config.filesystem.skip_directories))
        .collect();
    files.sort();
    files.dedup();
    check_with_files(root, config, &files)
}

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<super::super::analysis_policy::AnalysisDiagnostics> {
    let outputs: Result<Vec<_>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let options: Options = rule.try_rule_options()?;
            let files = super::super::selected_rule_files(root, config, rule, all_files)?;
            let audit = super::super::allow_audit::AllowAudit::new(
                root,
                config,
                rule,
                Some(sources),
                "intentionallyEmpty",
                options.intentionally_empty.len(),
            );
            let findings = scan(root, &options, &files, sources, &audit)?;
            super::super::file_diagnostics::application(
                root,
                sources,
                config,
                rule,
                findings,
                defer_suppression,
                super::super::file_diagnostics::InputUncertainty::Read,
            )
        })
        .collect();
    let mut diagnostics = super::super::analysis_policy::AnalysisDiagnostics::default();
    for output in outputs? {
        diagnostics.append(output);
    }
    Ok(super::super::file_diagnostics::sorted(diagnostics))
}
