#[test]
fn complete_check_fact_graph_adapter_reuses_exact_session_resolver_scope() {
    let root = fixture("dependents-basic");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..TsConfig::default()
    };
    let plan = GraphBuildPlan::imports_and_workspace();
    let (fact_plan, fact_context) = ts_fact_plan_and_context_for_plan(&root, plan);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files.clone(),
        crate::codebase::check_facts::CheckFactPlan {
            graph: fact_plan,
            graph_context: fact_context,
            ..Default::default()
        },
    );
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let build = || {
        DepGraph::build_with_complete_check_facts_and_session(
            CompleteCheckFactGraphBuildRequest {
                root: &root,
                tsconfig: &tsconfig,
                plan,
                files: files.clone(),
                config_path: None,
                facts: &facts,
            },
            session.clone(),
        )
        .expect("complete prepared facts build a graph")
    };

    build();
    let first = observer.snapshot().work;
    assert!(first["resolver.computations"] > 0);
    assert_eq!(
        first["resolver.computations"],
        first["resolver.unique_keys"]
    );

    build();
    let repeated = observer.snapshot().work;
    assert_eq!(
        repeated["resolver.computations"],
        first["resolver.computations"]
    );
    assert_eq!(
        repeated["resolver.unique_keys"],
        first["resolver.unique_keys"]
    );
    assert!(repeated["resolver.requests"] > first["resolver.requests"]);
    assert!(
        repeated["resolver.cache_hits"]
            > first
                .get("resolver.cache_hits")
                .copied()
                .unwrap_or_default()
    );
}
