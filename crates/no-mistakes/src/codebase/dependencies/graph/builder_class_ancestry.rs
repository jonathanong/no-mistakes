fn class_ancestry_for_graph(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    session: &crate::codebase::analysis_session::AnalysisSession,
) -> Arc<crate::codebase::analysis_session::class_ancestry::ClassAncestry> {
    use crate::codebase::analysis_session::class_ancestry::AncestryRequest;
    session.class_ancestry(AncestryRequest {
        root: inputs.root,
        resolve_bases: inputs.plan.extends,
        paths: inputs.graph_files.indexable(),
        facts,
        resolver,
        tsconfig: inputs.tsconfig,
        catalog: inputs.tsconfig_catalog,
    })
}
