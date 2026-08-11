use anyhow::Result;

pub(super) fn collect(
    enabled: bool,
    include_suppressed: bool,
    root: &std::path::Path,
    config: &no_mistakes::config::v2::NoMistakesConfig,
    files: &[std::path::PathBuf],
    sources: &no_mistakes::codebase::ts_source::SourceStore,
) -> Result<Vec<no_mistakes::codebase::rules::RuleFinding>> {
    if !enabled {
        return Ok(Vec::new());
    }
    if include_suppressed {
        no_mistakes::codebase::rules::agents_md_max_size::advisories_with_files_sources_and_deferred_suppression(root, config, files, sources)
    } else {
        no_mistakes::codebase::rules::agents_md_max_size::advisories_with_files_and_sources(
            root, config, files, sources,
        )
    }
}
