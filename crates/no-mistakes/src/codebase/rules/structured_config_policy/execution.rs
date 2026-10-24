use super::*;

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<super::super::analysis_policy::AnalysisDiagnostics> {
    super::super::file_diagnostics::read_applications::<Options>(
        super::super::file_diagnostics::FileRuleInputs {
            root,
            config,
            files: all_files,
            sources,
            defer_suppression,
        },
        RULE_ID,
        |options, files, scope, roots| scan(root, options, files, scope, roots, sources),
    )
}
