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
            let opts: Options = rule.try_rule_options()?;
            if !opts.configured() {
                return Ok(Vec::new());
            }
            let target_roots = super::super::target_roots(root, config, rule);
            let skip = super::super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|path| {
                    super::super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
                })
                .cloned()
                .collect();
            let mut files =
                super::super::path_filter::filter_rule_files(root, config, rule, &files)?;
            let filter = super::super::path_filter::RulePathFilter::new(root, config, rule)?;
            // Explicit paths are requested inputs even when missing from the inventory.
            for configured in [
                &opts.workspace_yaml,
                &opts.dependabot_path,
                &opts.lockfile_path,
            ]
            .into_iter()
            .flatten()
            {
                let path = root.join(configured);
                if filter.is_match(&path)
                    && super::super::file_allowed_by_roots_and_skip(
                        root,
                        &skip,
                        &path,
                        &target_roots,
                    )
                    && !files.contains(&path)
                {
                    files.push(path);
                }
            }
            Ok(scan::scan(root, &opts, &files, sources))
        },
    )
}
