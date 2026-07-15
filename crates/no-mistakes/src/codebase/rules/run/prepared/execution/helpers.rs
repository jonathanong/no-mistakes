use super::*;

pub(super) fn storybook_findings(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    tsconfig_path: Option<&Path>,
    prepared_tsconfig: &crate::codebase::ts_resolver::TsConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    session: &std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
) -> Result<Vec<RuleFinding>> {
    match inferred_roots {
        Some(inferred_roots) => {
            require_storybook_stories::check_with_prepared_facts_and_inferred_and_session(
                root,
                config,
                tsconfig_path,
                prepared_tsconfig,
                shared,
                inferred_roots,
                session,
            )
        }
        None => require_storybook_stories::check_with_prepared_facts_and_session(
            root,
            config,
            tsconfig_path,
            prepared_tsconfig,
            shared,
            session,
        ),
    }
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
