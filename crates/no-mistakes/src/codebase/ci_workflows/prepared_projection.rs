impl PreparedWorkflowTopologyInputs {
    /// Apply the loader's final selection stage to the already-resolved topology.
    pub(crate) fn project_topology(
        &self,
        filters: &[String],
    ) -> std::borrow::Cow<'_, WorkflowTopology> {
        use crate::codebase::workflow_topology::{
            model::DiagnosticCode, topology_graph, topology_identifiers,
        };
        if filters.is_empty() {
            return std::borrow::Cow::Borrowed(self.topology());
        }
        let topology = self.topology();
        let workflows = topology
            .workflows
            .iter()
            .map(|workflow| (workflow.path.clone(), workflow.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        let directories = self
            .ci
            .workflow_dirs
            .iter()
            .map(|directory| {
                crate::codebase::ci_graph::relative_slash(
                    &self.root,
                    &crate::codebase::ts_resolver::normalize_path(&self.root.join(directory)),
                )
            })
            .collect::<Vec<_>>();
        let mut diagnostics = topology.diagnostics.clone();
        topology_graph::diagnose_workflow_filters(
            filters,
            &workflows,
            &mut diagnostics,
            &directories,
        );
        let selected = topology_graph::select_workflow_paths(
            filters,
            &workflows,
            &topology.edges,
            &directories,
        );
        let mut result = topology.clone();
        result
            .workflows
            .retain(|workflow| selected.contains(&workflow.path));
        result.jobs.retain(|job| {
            selected.contains(topology_identifiers::workflow_path_from_id(
                &job.workflow_id,
            ))
        });
        result
            .edges
            .retain(|edge| topology_graph::edge_belongs_to_selection(edge, &selected));
        result.diagnostics = diagnostics
            .into_iter()
            .filter(|diagnostic| {
                diagnostic.code == DiagnosticCode::InvalidWorkflowFilter
                    || selected.contains(&diagnostic.workflow_path)
            })
            .collect();
        std::borrow::Cow::Owned(topology_graph::sort_topology(result))
    }
}
