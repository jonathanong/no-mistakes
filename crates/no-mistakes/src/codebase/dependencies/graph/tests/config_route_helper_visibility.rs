use super::*;

#[test]
fn injected_facts_cannot_resolve_route_helpers_outside_visible_files() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let plan = GraphBuildPlan {
        routes: true,
        ..GraphBuildPlan::default()
    };
    let discovered = GraphFiles::discover(&root).all().to_vec();
    let helper = root.join("src/entity-href.ts");
    let client = root.join("src/client.ts");
    let entity_route = root.join("backend/api/entity.mts");
    let fact_context = ts_fact_context_for_plan(&root, plan);
    let facts = collect_ts_facts_with_context(&discovered, plan.ts_fact_plan(), &fact_context);

    // Keep injected facts for the ignored helper to prove resolver visibility,
    // rather than fact availability, prevents the false route edge.
    assert!(facts.contains_key(&helper));
    let graph_files = GraphFiles::from_files(
        discovered
            .into_iter()
            .filter(|path| path != &helper)
            .collect(),
    );
    assert!(helper.exists());
    assert!(!graph_files.visible().contains(&helper));

    let graph = DepGraph::build_with_plan_files_config_and_facts(
        &root,
        &tsconfig,
        plan,
        &graph_files,
        None,
        Some(&facts),
    )
    .unwrap();
    let client = NodeId::File(client);
    assert!(graph
        .dependencies_of_node(&client)
        .into_iter()
        .flatten()
        .all(|(target, kind)| {
            *kind != EdgeKind::RouteRef || target.as_file() != Some(entity_route.as_path())
        }));
}
