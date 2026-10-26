use super::*;

#[test]
fn directory_roots_are_recursive_filtered_and_deduplicated() {
    let root = fixture("forbidden-dependencies-directory");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|f| (f.file.as_str(), f.target.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            ("web/app/nested/page.ts", Some("fs")),
            ("web/app/page.ts", Some("fs"))
        ],
    );
    assert!(findings
        .iter()
        .all(|f| f.message.contains(&format!("dependencies '{}'", f.file))));
}

#[test]
fn absolute_directory_roots_preserve_the_prepared_file_universe() {
    let root = fixture("forbidden-dependencies-directory");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let mut opts: Options = config.rules[0].try_rule_options().unwrap();
    opts.roots = vec![root.join("web/app").to_string_lossy().into_owned()];
    let tsconfig = resolve_tsconfig(&root, None).unwrap();
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    // The graph is deliberately broader than the request's inventory.
    let universe = [root.join("web/app/page.ts")]
        .into_iter()
        .collect::<crate::fx::PathSet>();
    let findings = check_rule_application(
        &root,
        &config,
        &config.rules[0],
        &opts,
        &graph,
        application_scope(None, Some(&universe), None),
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "web/app/page.ts");
    assert_eq!(findings[0].target.as_deref(), Some("fs"));
}

#[test]
fn directory_roots_share_prepared_facts_without_discovery_or_parsing() {
    let root = fixture("forbidden-dependencies-directory");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let standalone = check(&root, &config, None).unwrap();
    let plan = graph_plan(&config).unwrap();
    let (graph, graph_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan(&root, plan);
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            graph,
            graph_context,
            ..Default::default()
        },
    );
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    session.visible_paths(&root);
    let tsconfig = resolve_tsconfig(&root, None).unwrap();
    let prepared = shared_facts::check_with_prepared_facts_and_session(
        &root, &config, None, &tsconfig, &facts, None, &session,
    )
    .unwrap();
    assert_eq!(prepared, standalone);
    assert_eq!(prepared.len(), 2);
    let work = observer.snapshot().work;
    assert_eq!(work["discovery.roots"], 1, "{work:#?}");
    assert_eq!(work["graph.builds"], 1, "{work:#?}");
    assert_eq!(
        work.get("parse.files").copied().unwrap_or_default(),
        0,
        "{work:#?}"
    );
}
