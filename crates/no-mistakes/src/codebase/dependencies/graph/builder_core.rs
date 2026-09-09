impl DepGraph {
    fn build_with_plan_files_options_and_facts(
        edge_inputs: GraphEdgeBuildInputs<'_>,
        facts: Option<&dyn TsFactLookup>,
        supplied_fact_policy: SuppliedFactPolicy,
        session: std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
    ) -> Result<Self> {
        include!("builder_core/body.rs")
    }
}
