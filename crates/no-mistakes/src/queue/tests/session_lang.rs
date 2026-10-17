pub(crate) fn language_queue_sites_for_test(
    root: &std::path::Path,
    session: &crate::codebase::analysis_session::AnalysisSession,
    config: &crate::config::v2::NoMistakesConfig,
) {
    let _ = super::super::lang::language_queue_sites(root, session, config, None);
}
