impl SharedTraversalContext {
    pub(crate) fn signature_impact_json(
        &self,
        args: &crate::codebase::symbols::SymbolsArgs,
    ) -> Result<String> {
        let test_filter = self.test_filter.clone();
        let session = self.session.clone();
        let graph = self.graph_shared()?;
        crate::codebase::symbols::signature_impact_json_with_prepared(
            args,
            &self.root,
            crate::codebase::symbols::PreparedSignatureImpact {
                session: &session,
                tsconfig_catalog: &self.tsconfig_catalog,
                graph_files: &self.graph_files,
                test_filter: &test_filter,
                workspace: self.prepared_graph.workspace(),
                graph: graph.as_ref(),
                facts: self.prepared_facts(),
            },
        )
    }

    pub(crate) fn flow_report(
        &self,
        options: &crate::flow_query::FlowOptions,
    ) -> Result<crate::flow_query::FlowReport> {
        let graph = self.graph_shared()?;
        crate::flow_query::run_with_prepared_graph(options, &self.root, graph.as_ref())
    }

    pub(crate) fn effects_report(
        &self,
        selection: &crate::effects_query::EffectsSelection,
        entry: &Path,
        depth: Option<usize>,
    ) -> Result<crate::effects_query::EffectsReport> {
        let graph = self.graph_shared()?;
        crate::effects_query::run_with_prepared(
            &self.root,
            selection,
            entry,
            depth,
            graph.as_ref(),
            self.prepared_facts(),
        )
    }

    pub(crate) fn rsc_callers_report(
        &self,
        component: &Path,
        depth: Option<usize>,
    ) -> Result<crate::rsc_callers_query::RscCallersReport> {
        let graph = self.graph_shared()?;
        crate::rsc_callers_query::run_with_prepared(
            &self.root,
            component,
            depth,
            graph.as_ref(),
            self.prepared_facts(),
        )
    }
}
