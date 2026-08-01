use super::*;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/graph/workflow-topology-prepared"),
    )
}

#[test]
fn prepared_workflows_are_projected_to_the_graph_workflow_universe() {
    let root = fixture_root();
    let workflow = root.join(".github/workflows/ci.yml");
    let prepared =
        crate::codebase::ci_workflows::ParsedWorkflowSet::from_paths(&root, [workflow.clone()]);

    let projected = parsed_workflows_for_graph(
        &root,
        &[workflow],
        &crate::config::v2::schema::CiConfig::default(),
        Some(&prepared),
    );

    assert_eq!(projected.documents.len(), 1);
    assert_eq!(projected.documents[0].path, ".github/workflows/ci.yml");
}
