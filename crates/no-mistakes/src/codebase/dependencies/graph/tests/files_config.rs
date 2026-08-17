use super::*;

#[test]
fn files_config_session_does_not_discover_twice() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let snapshot = session.visible_paths(&root);
    assert_eq!(observer.snapshot().work["discovery.roots"], 1);

    let from_session =
        graph_config_options_with_config_and_session(&root, None, Some(&session), None);
    assert!(from_session.is_some());
    assert_eq!(observer.snapshot().work["discovery.roots"], 1);

    let from_snapshot = graph_config_options_with_config_and_session(
        &root,
        None,
        Some(&session),
        Some(snapshot.paths_for(&root).as_ref()),
    );
    assert!(from_snapshot.is_some());
    assert_eq!(observer.snapshot().work["discovery.roots"], 1);
}

#[test]
fn files_config_complete_graph_build_does_not_repeat_work() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let snapshot = session.visible_paths(&root);
    let files = snapshot.paths_for(&root).as_ref().clone();
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let tsconfig_catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig.clone(), None);
    let plan = GraphBuildPlan {
        imports: true,
        routes: true,
        ..GraphBuildPlan::default()
    };
    let (fact_plan, fact_context) = ts_fact_plan_and_context_for_plan_with_config_and_session(
        &root,
        plan,
        None,
        Some(&session),
        Some(&files),
    );

    crate::ast::with_request_parse_cache(|| {
        let facts = crate::codebase::check_facts::collect_check_facts(
            &root,
            files.clone(),
            crate::codebase::check_facts::CheckFactPlan {
                graph: fact_plan,
                graph_context: fact_context,
                ..Default::default()
            },
        );
        DepGraph::build_with_complete_check_facts_and_session(
            CompleteCheckFactGraphBuildRequest {
                root: &root,
                tsconfig: &tsconfig,
                tsconfig_catalog: &tsconfig_catalog,
                plan,
                files,
                config_path: None,
                facts: &facts,
            },
            session.clone(),
        )
        .expect("complete prepared facts build a graph");

        let work = observer.snapshot().work;
        assert_eq!(work["discovery.roots"], 1, "{work:#?}");
        assert_eq!(work["graph.builds"], 1, "{work:#?}");
        // Fact collection uses a disabled session. The graph build must reuse
        // those facts instead of parsing the request universe a second time.
        assert_eq!(
            work.get("parse.files").copied().unwrap_or_default(),
            0,
            "{work:#?}"
        );
    });
}

#[test]
fn files_config_session_helper_does_not_call_discover_when_snapshot_exists() {
    let source = include_str!("../files_config_session.rs");
    assert_eq!(
        source.matches("discover_visible_paths").count(),
        1,
        "standalone fallback is the only discover_visible_paths call"
    );
    assert!(source.contains("graph_config_options_with_config_and_session"));
    assert!(source.contains("session.visible_paths(root)"));
    assert!(!include_str!("../files_config.rs").contains("discover_visible_paths"));
}
