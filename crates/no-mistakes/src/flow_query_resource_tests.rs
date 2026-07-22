use super::*;

fn resource_fixture() -> tempfile::TempDir {
    crate::test_support::materialize_saved_fixture(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/test-plan/resource-impact"),
    )
}

#[test]
fn public_flow_query_keeps_tracked_resources_under_source_skipped_directories() {
    let fixture = resource_fixture();
    let root = fixture.path().canonicalize().unwrap();
    let report = run(&FlowOptions {
        target: "skipped-resource-consumer.ts".to_string(),
        root,
        tsconfig: None,
        config: None,
        direction: FlowDirection::Deps,
        depth: 1,
        relationships: vec![RelationshipArg::Resource],
    })
    .unwrap();

    assert!(report.nodes.iter().any(|node| {
        node.kind == "file" && node.file.as_deref() == Some("fixtures/schema.sql")
    }));
    assert!(report.edges.iter().any(|edge| {
        edge.from == "skipped-resource-consumer.ts"
            && edge.to == "fixtures/schema.sql"
            && edge.kind == "resource"
    }));
}
