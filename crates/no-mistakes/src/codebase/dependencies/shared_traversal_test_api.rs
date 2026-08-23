use super::*;

impl SharedTraversalContext {
    pub(crate) fn facts(&mut self) -> &crate::codebase::ts_source::facts::TsFactMap {
        self.ensure_facts();
        self.facts.as_ref().expect("TS facts are initialized")
    }

    pub(crate) fn graph(&mut self) -> Result<&graph::DepGraph> {
        if self.graph.is_none() {
            self.graph = Some(self.request_graph(self.build_plan)?);
        }
        self.graph
            .as_deref()
            .context("dependency graph was not initialized")
    }

    pub(crate) fn request_graph(
        &mut self,
        plan: graph::GraphBuildPlan,
    ) -> Result<std::sync::Arc<graph::DepGraph>> {
        self.ensure_facts();
        let graph = self.request_graph_shared(plan)?;
        self.graph_builds = self.graph_cache.build_count();
        Ok(graph)
    }

    pub(crate) fn canonical_graph(&mut self) -> Result<std::sync::Arc<graph::DepGraph>> {
        self.request_graph(self.build_plan)
    }

    pub(crate) fn graph_build_count(&self) -> usize {
        self.graph_cache.build_count()
    }
}

#[test]
fn prepared_traversal_isolates_provenance_with_compute() {
    let collect = include_str!("shared_traversal_collect.rs");
    let compute = collect
        .split("cached_traversal_entries(")
        .nth(1)
        .expect("prepared collect caches traversal entries");
    assert!(
        compute.contains("provenance_for"),
        "provenance selection must run inside the isolated cache compute"
    );
}
