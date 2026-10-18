fn workflow_topology_for_graph<'a>(
    root: &Path,
    ci: &crate::config::v2::schema::CiConfig,
    parsed: &crate::codebase::ci_workflows::ParsedWorkflowSet,
    prepared: Option<crate::codebase::ci_workflows::PreparedWorkflowDocuments<'a>>,
) -> std::borrow::Cow<'a, crate::codebase::workflow_topology::model::WorkflowTopology> {
    use crate::codebase::ci_workflows::PreparedWorkflowDocuments;
    if let Some(PreparedWorkflowDocuments::Topology(inputs)) = prepared {
        if inputs.matches_projection(root, ci, parsed) {
            return std::borrow::Cow::Borrowed(inputs.topology());
        }
    }
    std::borrow::Cow::Owned(
        crate::codebase::workflow_topology::load_workflow_topology_from_parsed(
            root,
            ci,
            parsed,
            &[],
        ),
    )
}

#[cfg(test)]
#[path = "builder_workflow_topology_tests.rs"]
mod prepared_workflow_topology_tests;
