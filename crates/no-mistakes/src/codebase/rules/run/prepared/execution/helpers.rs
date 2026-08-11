use super::*;

pub(super) struct StorybookFindingsRequest<'a> {
    pub(super) root: &'a Path,
    pub(super) config: &'a crate::config::v2::NoMistakesConfig,
    pub(super) prepared_tsconfig_catalog: &'a crate::codebase::ts_resolver::TsConfigCatalog,
    pub(super) shared: &'a crate::codebase::check_facts::CheckFactMap,
    pub(super) inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
    pub(super) session: &'a std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    pub(super) defer_suppression: bool,
    pub(super) sources: &'a crate::codebase::ts_source::SourceStore,
}

pub(super) fn storybook_findings(input: StorybookFindingsRequest<'_>) -> Result<Vec<RuleFinding>> {
    let StorybookFindingsRequest {
        root,
        config,
        prepared_tsconfig_catalog,
        shared,
        inferred_roots,
        session,
        defer_suppression,
        sources,
    } = input;
    require_storybook_stories::check_with_prepared_facts_for_aggregate(
        require_storybook_stories::PreparedStorybookCheck {
            root,
            config,
            prepared_tsconfig_catalog,
            shared,
            inferred_roots,
            session,
            defer_suppression,
            sources,
        },
    )
}

pub(super) fn suppress_findings(
    root: &Path,
    findings: &mut Vec<RuleFinding>,
    sources: &crate::codebase::ts_source::SourceStore,
) {
    suppress_rule_findings_with_sources(root, findings, sources);
}

pub(super) fn finalize_findings(
    findings: Vec<RuleFinding>,
    suppression_sources: Vec<Option<String>>,
) -> PreparedRuleFindings {
    let mut paired = findings
        .into_iter()
        .zip(suppression_sources)
        .collect::<Vec<_>>();
    paired.sort_by(|(a, _), (b, _)| a.cmp(b));
    paired.dedup_by(|(a, source_a), (b, source_b)| a == b && source_a == source_b);
    let (findings, suppression_sources): (Vec<_>, Vec<_>) = paired.into_iter().unzip();
    PreparedRuleFindings {
        findings,
        suppression_sources,
    }
}
