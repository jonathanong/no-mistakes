impl DepGraph {
    /// Attach compact Vitest ownership relationships to an already-built
    /// canonical graph. Expansion is deferred until adjacency is requested.
    pub(crate) fn with_vitest_setup_projects(
        mut self,
        projects: Vec<VitestSetupProject>,
    ) -> Self {
        self.vitest_setup_projects = projects;
        self.effective_edges = OnceLock::new();
        self
    }

    fn materialize_vitest_setup_edges(&self) -> Vec<CanonicalEdge<NodeId, EdgeKind>> {
        let mut edges = Vec::new();
        for node in self.edges.forward().keys() {
            let NodeId::File(test) = node else {
                continue;
            };
            let relative = crate::codebase::ts_source::relative_slash_path(&self.root, test);
            let matched = self
                .vitest_setup_projects
                .iter()
                .filter(|project| project.filter.is_match(&relative))
                .collect::<Vec<_>>();
            for project in matched
                .iter()
                .copied()
                .filter(|project| !vitest_project_is_dominated(project, &matched))
            {
                edges.extend(project.setups.iter().map(|(setup, field)| {
                    CanonicalEdge::new(
                        NodeId::File(test.clone()),
                        NodeId::File(setup.clone()),
                        EdgeKind::VitestSetup(*field),
                    )
                }));
            }
        }
        edges
    }
}

fn vitest_project_is_dominated(
    project: &VitestSetupProject,
    matched: &[&VitestSetupProject],
) -> bool {
    let Some(scope) = project.scope.as_deref() else {
        return false;
    };
    matched.iter().any(|other| {
        other.config != project.config
            && other
                .scope
                .as_deref()
                .is_some_and(|other_scope| vitest_scope_is_descendant(scope, other_scope))
    })
}

fn vitest_scope_is_descendant(ancestor: &str, descendant: &str) -> bool {
    let ancestor = if ancestor == "." { "" } else { ancestor };
    let descendant = if descendant == "." { "" } else { descendant };
    ancestor != descendant
        && if ancestor.is_empty() {
            !descendant.is_empty()
        } else {
            descendant
                .strip_prefix(ancestor)
                .is_some_and(|rest| rest.starts_with('/'))
        }
}
