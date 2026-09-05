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
fn files_config_session_builder_reuses_prepared_test_filter() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let files = session.visible_paths(&root).paths_for(&root).as_ref().clone();
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let graph_files = GraphFiles::from_files(files);
    let plan = GraphBuildPlan {
        imports: true,
        routes: true,
        ..GraphBuildPlan::default()
    };
    crate::ast::with_request_parse_cache(|| {
        DepGraph::build_with_plan_files_config_facts_and_session(
            &root,
            &tsconfig,
            plan,
            &graph_files,
            None,
            None,
            session.clone(),
        )
        .expect("session graph builder builds a graph");
        let work = observer.snapshot().work;
        assert_eq!(work["graph.builds"], 1, "{work:#?}");
        assert_eq!(work["test_filter.builds"], 1, "{work:#?}");

        DepGraph::build_with_plan_files_config_facts_and_session(
            &root,
            &tsconfig,
            plan,
            &graph_files,
            None,
            None,
            session,
        )
        .expect("second session graph builder reuses the filter");
        let work = observer.snapshot().work;
        assert_eq!(work["test_filter.builds"], 1, "{work:#?}");
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
    assert!(source.contains("graph_config_options_from_loaded_with_test_filter"));
    assert!(source.contains("session.test_file_filter_with_visible("));
    assert!(!source.contains("TestFileFilter::new("));
    assert!(!include_str!("../files_config.rs").contains("discover_visible_paths"));
}

/// Reuse must not increment `test_filter.project_filters` or `test_filter.builds`.
#[test]
fn files_config_session_does_not_rebuild_prepared_test_filter() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        let session =
            crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
        let snapshot = session.visible_paths(&root);
        let v2 = crate::config::v2::load_v2_config(&root, None).unwrap();
        session.insert_test_file_filter(
            &root,
            crate::codebase::test_filter::TestFileFilter::from_prepared_projects(
                &root,
                &v2,
                snapshot.paths_for(&root).as_ref(),
                Vec::new(),
            ),
        );
        session.insert_test_file_filter(
            &root,
            crate::codebase::test_filter::TestFileFilter::fallback_only(),
        );

        let project_filters_before = observer
            .snapshot()
            .work
            .get("test_filter.project_filters")
            .copied()
            .unwrap_or_default();
        let builds_before = observer
            .snapshot()
            .work
            .get("test_filter.builds")
            .copied()
            .unwrap_or_default();

        let from_session =
            graph_config_options_with_config_and_session(&root, None, Some(&session), None);
        assert!(from_session.is_some());
        let from_snapshot = graph_config_options_with_config_and_session(
            &root,
            None,
            Some(&session),
            Some(snapshot.paths_for(&root).as_ref()),
        );
        assert!(from_snapshot.is_some());

        let work = observer.snapshot().work;
        assert_eq!(
            work.get("test_filter.project_filters")
                .copied()
                .unwrap_or_default(),
            project_filters_before,
            "prepared session must not invoke project_filters during graph config, {work:#?}"
        );
        assert_eq!(
            work.get("test_filter.builds").copied().unwrap_or_default(),
            builds_before,
            "prepared session must not compile a second TestFileFilter, {work:#?}"
        );
        assert_eq!(project_filters_before, 0, "{work:#?}");
        assert_eq!(builds_before, 0, "{work:#?}");
    });
}

#[test]
fn files_config_session_builds_test_filter_once() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        let session =
            crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));

        let first =
            graph_config_options_with_config_and_session(&root, None, Some(&session), None);
        assert!(first.is_some());
        let work = observer.snapshot().work;
        let project_filters = work
            .get("test_filter.project_filters")
            .copied()
            .unwrap_or_default();
        assert_eq!(work["test_filter.builds"], 1, "{work:#?}");
        assert!(
            project_filters >= 1,
            "first session graph config compiles project filters, {work:#?}"
        );

        let second =
            graph_config_options_with_config_and_session(&root, None, Some(&session), None);
        assert!(second.is_some());
        let work = observer.snapshot().work;
        assert_eq!(work["test_filter.builds"], 1, "{work:#?}");
        assert_eq!(
            work.get("test_filter.project_filters")
                .copied()
                .unwrap_or_default(),
            project_filters,
            "second graph config must not compile project filters again, {work:#?}"
        );
    });
}

#[test]
fn files_config_session_test_filters_are_independent_per_root() {
    let root_a =
        crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let root_b =
        crate::codebase::ts_resolver::normalize_path(&fixture("graph-project-route-config"));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        let session =
            crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
        let snapshot_a = session.visible_paths(&root_a);
        let v2_a = crate::config::v2::load_v2_config(&root_a, None).unwrap();
        session.insert_test_file_filter(
            &root_a,
            crate::codebase::test_filter::TestFileFilter::from_prepared_projects(
                &root_a,
                &v2_a,
                snapshot_a.paths_for(&root_a).as_ref(),
                Vec::new(),
            ),
        );
        assert_eq!(work_count(&observer, "test_filter.builds"), 0);
        assert_eq!(work_count(&observer, "test_filter.project_filters"), 0);

        let first_b =
            graph_config_options_with_config_and_session(&root_b, None, Some(&session), None);
        assert!(first_b.is_some());
        let after_b = observer.snapshot().work;
        assert_eq!(after_b["test_filter.builds"], 1, "{after_b:#?}");
        let project_filters = after_b
            .get("test_filter.project_filters")
            .copied()
            .unwrap_or_default();
        assert!(
            project_filters >= 1,
            "unseeded root B must compile project filters, {after_b:#?}"
        );

        let second_a =
            graph_config_options_with_config_and_session(&root_a, None, Some(&session), None);
        assert!(second_a.is_some());
        let second_b =
            graph_config_options_with_config_and_session(&root_b, None, Some(&session), None);
        assert!(second_b.is_some());
        let work = observer.snapshot().work;
        assert_eq!(work["test_filter.builds"], 1, "{work:#?}");
        assert_eq!(
            work.get("test_filter.project_filters")
                .copied()
                .unwrap_or_default(),
            project_filters,
            "second loads must not compile project filters again, {work:#?}"
        );
    });
}

#[test]
fn files_config_unprepared_graph_config_compiles_project_filters_without_session_build() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        let options = graph_config_options_with_config_and_session(&root, None, None, None);
        assert!(options.is_some());
        let work = observer.snapshot().work;
        assert_eq!(
            work.get("test_filter.builds").copied().unwrap_or_default(),
            0,
            "{work:#?}"
        );
        assert!(
            work.get("test_filter.project_filters")
                .copied()
                .unwrap_or_default()
                >= 1,
            "session=None must compile via TestFileFilter::new, {work:#?}"
        );
    });

    let unprepared = include_str!("../builder.rs")
        .split("pub(crate) fn build_with_plan_files_config_and_facts(")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(crate) fn build_with_plan_files_config_facts_and_session(")
                .next()
        })
        .expect("unprepared graph builder");
    let call = unprepared
        .split("graph_config_options_for_plan_with_config_and_session(")
        .nth(1)
        .expect("unprepared graph config load");
    assert!(
        call.contains("None,"),
        "standalone graph config must pass session=None"
    );
}

fn work_count(observer: &crate::diagnostics::InvocationObserver, metric: &str) -> u64 {
    observer
        .snapshot()
        .work
        .get(metric)
        .copied()
        .unwrap_or_default()
}

#[test]
fn graph_config_options_from_loaded_builds_the_unprepared_test_filter() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let config = crate::codebase::config::load_config(&root).unwrap();
    let v2 = crate::config::v2::load_v2_config(&root, None).unwrap();
    let paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let options = graph_config_options_from_loaded(&root, &config, &v2, &paths);
    assert!(options.test_filter.is_some());

    let rule = crate::config::v2::schema::RewriteRule {
        source: "/from".into(),
        destination: "/to".into(),
    };
    let other = crate::config::v2::schema::RewriteRule {
        source: "/other".into(),
        destination: "/dest".into(),
    };
    assert_eq!(
        dedup_rewrites(vec![rule.clone(), other.clone(), rule]),
        vec![
            crate::config::v2::schema::RewriteRule {
                source: "/from".into(),
                destination: "/to".into(),
            },
            other,
        ]
    );
}

#[test]
fn graph_config_options_for_plan_skip_plans_that_do_not_need_config() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let imports_only = GraphBuildPlan {
        imports: true,
        ..GraphBuildPlan::default()
    };
    assert!(graph_config_options_for_plan_with_config(&root, imports_only, None).is_none());
    let routes = GraphBuildPlan {
        routes: true,
        ..GraphBuildPlan::default()
    };
    assert!(graph_config_options_for_plan_with_config(&root, routes, None).is_some());
    let config = root.join(".no-mistakes.yml");
    assert!(graph_config_options_for_plan_with_config(&root, routes, Some(&config)).is_some());
}
