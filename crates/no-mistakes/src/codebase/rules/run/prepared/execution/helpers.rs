use super::*;

pub(super) fn storybook_findings(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    prepared_tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
    shared: &crate::codebase::check_facts::CheckFactMap,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    session: &std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    require_storybook_stories::check_with_prepared_facts_for_aggregate(
        root,
        config,
        prepared_tsconfig_catalog,
        shared,
        inferred_roots,
        session,
        defer_suppression,
    )
}

pub(super) fn suppress_findings(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) {
    match sources {
        Some(sources) => suppress_rule_findings_with_sources(root, findings, sources),
        None => suppress_rule_findings(root, findings),
    }
}
