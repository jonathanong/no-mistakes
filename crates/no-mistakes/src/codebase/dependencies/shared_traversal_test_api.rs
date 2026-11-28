use super::*;

impl SharedTraversalContext {
    pub fn facts(&mut self) -> &crate::codebase::ts_source::facts::TsFactMap {
        self.ensure_facts();
        self.facts.as_ref().expect("TS facts are initialized")
    }

    pub fn graph(&mut self) -> Result<&graph::DepGraph> {
        if self.graph.is_none() {
            self.graph = Some(self.request_graph(self.build_plan)?);
        }
        self.graph
            .as_deref()
            .context("dependency graph was not initialized")
    }

    pub fn request_graph(
        &mut self,
        plan: graph::GraphBuildPlan,
    ) -> Result<std::sync::Arc<graph::DepGraph>> {
        self.ensure_facts();
        let graph = self.request_graph_shared(plan)?;
        self.graph_builds = self.graph_cache.build_count();
        Ok(graph)
    }

    pub fn canonical_graph(&mut self) -> Result<std::sync::Arc<graph::DepGraph>> {
        self.request_graph(self.build_plan)
    }

    pub fn graph_build_count(&self) -> usize {
        self.graph_cache.build_count()
    }
}
