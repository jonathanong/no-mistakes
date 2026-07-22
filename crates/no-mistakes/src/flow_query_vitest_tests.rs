use super::*;

#[test]
fn flow_query_reports_resolved_vitest_setup_edges() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    );
    let report = run(&FlowOptions {
        target: "setup/conditional-a.ts".to_string(),
        root,
        tsconfig: None,
        config: None,
        direction: FlowDirection::Dependents,
        depth: 1,
        relationships: vec![RelationshipArg::Test],
    })
    .unwrap();

    assert!(
        report.edges.iter().any(|edge| {
            edge.from == "conditional-owner/conditional.test.ts"
                && edge.to == "setup/conditional-a.ts"
                && edge.kind == "vitest-setup"
        }),
        "{report:#?}"
    );
}
