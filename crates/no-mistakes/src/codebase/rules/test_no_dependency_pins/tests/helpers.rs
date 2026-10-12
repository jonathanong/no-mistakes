use super::*;

pub(in crate::codebase::rules::test_no_dependency_pins) fn scan_with_sources(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(sources.observer().cloned());
    super::super::projection::scan_with_sources_and_facts(
        root,
        opts,
        files,
        sources,
        None,
        Some(&session),
    )
}

pub(in crate::codebase::rules::test_no_dependency_pins) fn check_source(
    file: &str,
    content: &str,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let ranges = super::super::scan::jsx_text_ranges::collect(
        std::path::Path::new(file),
        &std::sync::Arc::from(content),
        &session,
    );
    super::super::scan::check_source_with_ranges(file, content, opts, &ranges)
}
