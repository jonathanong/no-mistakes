fn class_ancestry_for_graph(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    session: &crate::codebase::analysis_session::AnalysisSession,
    constructors: &[ResolvedCallSite],
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> Arc<crate::codebase::analysis_session::class_ancestry::ClassAncestry> {
    use crate::codebase::analysis_session::class_ancestry::AncestryRequest;
    let resolver = crate::codebase::ts_resolver::WorkspaceFallbackResolver::new(
        resolver,
        workspace,
        inputs.graph_files,
    );
    session.class_ancestry(AncestryRequest {
        root: inputs.root,
        resolve_bases: inputs.plan.extends,
        paths: inputs.graph_files.indexable(),
        facts,
        resolver: &resolver,
        tsconfig: inputs.tsconfig,
        catalog: inputs.tsconfig_catalog,
        constructors,
        workspace,
    })
}
