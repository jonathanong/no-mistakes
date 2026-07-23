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

#[test]
fn import_only_flow_keeps_explicit_vitest_config_indexable() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    );
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let report = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        run(&FlowOptions {
            target: "vitest.config.ts".to_string(),
            root: root.clone(),
            tsconfig: None,
            config: None,
            direction: FlowDirection::Deps,
            depth: 1,
            relationships: vec![RelationshipArg::Import],
        })
        .unwrap()
    };

    assert!(
        report.edges.iter().any(|edge| {
            edge.from == "vitest.config.ts"
                && edge.to == "config/setup-selector.ts"
                && edge.kind == "import"
        }),
        "{report:#?}"
    );
    let work = observer.snapshot().work;
    assert_eq!(work["discovery.roots"], 1, "{work:#?}");
    assert_eq!(
        observer.source_read_snapshot()[&root.join("vitest.config.ts")],
        1
    );
}
