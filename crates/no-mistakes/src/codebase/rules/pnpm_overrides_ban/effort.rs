use super::*;

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, all_files, sources, defer_suppression)
        .map(|diagnostics| diagnostics.findings)
}

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<super::super::analysis_policy::AnalysisDiagnostics> {
    super::super::manifest_analysis::collect(
        root,
        config,
        RULE_ID,
        sources,
        defer_suppression,
        |rule| {
            let target_roots = super::super::target_roots(root, config, rule);
            let skip = super::super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|path| {
                    super::super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
                })
                .cloned()
                .collect();
            let files = super::super::path_filter::filter_rule_files(root, config, rule, &files)?;
            Ok(scan::scan(root, &files, sources))
        },
    )
}
