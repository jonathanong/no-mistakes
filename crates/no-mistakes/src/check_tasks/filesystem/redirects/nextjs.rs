use super::*;

pub(super) fn run(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    prepared: &rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    if let Some(resolver) = prepared.next_config_resolver {
        no_mistakes::perf_trace::rule(
            "analysis.filesystem_rules.nextjs-redirect-destinations",
            || {
                rules::nextjs_redirect_destinations::diagnostics_prepared(
                    root,
                    config,
                    files,
                    &prepared.sources,
                    Some(prepared.snapshot),
                    resolver,
                    defer_suppression,
                )
            },
        )
    } else {
        // Legacy standalone callers own missing preparation at this boundary.
        no_mistakes::perf_trace::rule(
            "analysis.filesystem_rules.nextjs-redirect-destinations",
            || {
                rules::nextjs_redirect_destinations::diagnostics_with_files_and_sources(
                    root,
                    config,
                    files,
                    &prepared.sources,
                    Some(prepared.snapshot),
                    defer_suppression,
                )
            },
        )
    }
}
