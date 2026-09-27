#[test]
fn integration_routes_shared_traversal_retains_real_runner_ownership() {
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::codebase::dependencies::{graph, SharedTraversalContext};
    let root = root();
    let mut shared = SharedTraversalContext::prepare(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan {
            imports: true,
            playwright_routes: true,
            ..Default::default()
        },
    )
    .unwrap();
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);
    let settings = settings(&snapshot);
    let source = &settings.route_coverage_sources[0];
    let links = shared
        .facts()
        .integration_route_links(source)
        .unwrap()
        .as_ref()
        .unwrap();
    assert!(links.iter().any(|link| link.occurrence.value == "/healthz"
        && link.entry_file == root.join("integration/web.test.ts")));
    let graph = shared.canonical_graph().unwrap();
    assert!(graph
        .dependencies_of_node(&graph::NodeId::file(root.join("integration/web.test.ts")))
        .unwrap()
        .iter()
        .any(|(target, kind)| {
            *kind == graph::EdgeKind::RouteTest
                && target.as_file() == Some(root.join("web/app/healthz/page.tsx").as_path())
        }));
}

#[test]
fn integration_routes_require_prepared_ownership_context() {
    let root = root();
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);
    let settings = settings(&snapshot);
    let plan = crate::playwright::analysis::pipeline_facts::standalone_fact_plan(
        &root,
        &settings,
        Default::default(),
        &snapshot,
    )
    .unwrap();
    let facts = crate::playwright::analysis::pipeline_facts::standalone_facts(
        &root,
        &settings,
        Default::default(),
        &snapshot,
    )
    .unwrap();
    let files = crate::codebase::ts_source::FileIdMap::default();
    let links = super::links::prepare_links(
        &root,
        &Default::default(),
        &plan,
        &files,
        Default::default(),
    );
    assert!(links.values().all(|result| result
        .as_ref()
        .unwrap_err()
        .contains("prepared Vitest runner configuration")));
    let error = super::edges(&root, &settings, &[], None).err().unwrap();
    assert!(error.to_string().contains("not a canonical route"));
    let routes = crate::playwright::routes::collect_routes_from_snapshot(
        &root.join(&settings.frontend_root),
        &snapshot,
    );
    assert!(super::edges(&root, &settings, &routes, None)
        .err()
        .unwrap()
        .to_string()
        .contains("prepared integration route facts"));
    let sources = snapshot.source_store_for(&root);
    let mut demand = crate::codebase::check_facts::CheckFactPlan::default();
    super::runner_plan(&root, &mut demand, &plan, &sources);
    assert!(
        demand.integration_runner_configs.is_none(),
        "an unconfigured resolver cannot own runner preparation"
    );
    let paths = snapshot.paths_for(&root);
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
        None, &root, &paths, &sources,
    )
    .unwrap();
    let mut configured = plan.clone();
    configured.configure_module_resolution(
        std::sync::Arc::new(tsconfig),
        std::sync::Arc::new(Default::default()),
        &snapshot,
        &root,
    );
    super::runner_plan(&root, &mut demand, &configured, &sources);
    let links = super::links::prepare_links(
        &root,
        &demand,
        &plan,
        &files,
        facts.integration_runner_configs.clone(),
    );
    assert!(links.values().all(|result| result
        .as_ref()
        .unwrap_err()
        .contains("prepared module resolution")));
    let mut files = files;
    let entry = root.join("integration/web.test.ts");
    files.insert(
        entry.clone(),
        crate::codebase::check_facts::CheckFileFacts {
            ts: facts.ts.get(&entry).unwrap().ts.clone(),
            ..Default::default()
        },
    );
    let sparse = super::links::prepare_links(
        &root,
        &demand,
        &configured,
        &files,
        facts.integration_runner_configs.clone(),
    );
    assert!(
        sparse
            .values()
            .all(|result| result.as_ref().unwrap().is_empty()),
        "unprepared imported declarations cannot claim routes"
    );
    files.insert(
        entry.clone(),
        crate::codebase::check_facts::CheckFileFacts {
            parse_error: Some("invalid registered module".into()),
            ..Default::default()
        },
    );
    let links = super::links::prepare_links(
        &root,
        &demand,
        &configured,
        &files,
        facts.integration_runner_configs,
    );
    assert!(links
        .values()
        .all(|result| result.as_ref().unwrap_err().contains(&format!(
            "failed to parse registered integration module {}",
            entry.display()
        ))));
}

include!("coverage_lookup_tests.rs");
