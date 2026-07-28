fn graph_import_resolver<'a>(
    edge_inputs: &'a GraphEdgeBuildInputs<'a>,
    session: &'a crate::codebase::analysis_session::AnalysisSession,
) -> crate::codebase::ts_resolver::ProjectImportResolver<'a> {
    crate::codebase::ts_resolver::ProjectImportResolver::new(
        edge_inputs.tsconfig,
        edge_inputs.tsconfig_catalog,
        edge_inputs.graph_files.visible(),
        edge_inputs.import_resolution_cache,
        session,
    )
}
