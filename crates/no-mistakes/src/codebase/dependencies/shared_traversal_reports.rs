impl SharedTraversalContext {
    pub(crate) fn signature_impact_json(
        &mut self,
        args: &crate::codebase::symbols::SymbolsArgs,
    ) -> Result<String> {
        let test_filter = self.test_filter.clone();
        let session = self.session.clone();
        self.graph()?;
        let graph = self
            .graph
            .as_ref()
            .context("dependency graph was not initialized")?;
        let facts = self
            .facts
            .as_ref()
            .context("TS facts were not initialized")?;
        crate::codebase::symbols::signature_impact_json_with_prepared(
            args,
            &self.root,
            &self.tsconfig,
            crate::codebase::symbols::PreparedSignatureImpact {
                session: &session,
                graph_files: &self.graph_files,
                test_filter: &test_filter,
                graph,
                facts,
            },
        )
    }

    pub(crate) fn flow_report(
        &mut self,
        options: &crate::flow_query::FlowOptions,
    ) -> Result<crate::flow_query::FlowReport> {
        self.graph()?;
        let graph = self
            .graph
            .as_ref()
            .context("dependency graph was not initialized")?;
        crate::flow_query::run_with_prepared_graph(options, &self.root, graph)
    }

    pub(crate) fn effects_report(
        &mut self,
        selection: &crate::effects_query::EffectsSelection,
        entry: &Path,
        depth: Option<usize>,
    ) -> Result<crate::effects_query::EffectsReport> {
        self.graph()?;
        let graph = self
            .graph
            .as_ref()
            .context("dependency graph was not initialized")?;
        let facts = self
            .facts
            .as_ref()
            .context("TS facts were not initialized")?;
        crate::effects_query::run_with_prepared(
            &self.root,
            selection,
            entry,
            depth,
            graph,
            facts,
        )
    }

    pub(crate) fn rsc_callers_report(
        &mut self,
        component: &Path,
        depth: Option<usize>,
    ) -> Result<crate::rsc_callers_query::RscCallersReport> {
        self.graph()?;
        let graph = self
            .graph
            .as_ref()
            .context("dependency graph was not initialized")?;
        let facts = self
            .facts
            .as_ref()
            .context("TS facts were not initialized")?;
        crate::rsc_callers_query::run_with_prepared(
            &self.root,
            component,
            depth,
            graph,
            facts,
        )
    }
}
