fn trpc_graph(name: &str, plan: GraphBuildPlan) -> (PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(name));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let graph = DepGraph::build_with_plan(&root, &tsconfig, plan).unwrap();
    (root, graph)
}

#[test]
fn graph_all_plan_keeps_trpc_opt_in() {
    assert!(!GraphBuildPlan::all().trpc);
    let allowed = [EdgeKind::TrpcCall, EdgeKind::TrpcProcedure]
        .into_iter()
        .collect();
    let plan = GraphBuildPlan::from_allowed(Some(&allowed));
    assert!(plan.trpc);
    assert!(!plan.imports);
    assert!(!plan.queues);
}

#[test]
fn configured_trpc_calls_hop_through_virtual_procedure_nodes() {
    let (root, graph) = trpc_graph(
        "trpc-basic",
        GraphBuildPlan {
            imports: true,
            trpc: true,
            ..GraphBuildPlan::default()
        },
    );
    let client = root.join("src/client.ts");
    let router = root.join("src/router.ts");
    let procedure = NodeId::trpc_procedure(&router, "user.get");

    let calls = graph.deps_of(
        &[NodeId::file(&client)],
        None,
        Some(&[EdgeKind::TrpcCall].into()),
    );
    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::TrpcProcedure { router_file, procedure }
                if router_file.as_ref() == router.as_path() && procedure.as_ref() == "user.get"
        )
    }));
    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::TrpcProcedure { procedure, .. } if procedure.as_ref() == "user.create"
        )
    }));
    assert_eq!(
        procedure.display_name(&root),
        "src/router.ts#procedure:user.get"
    );

    let routers = graph.deps_of(&[procedure], None, Some(&[EdgeKind::TrpcProcedure].into()));
    assert!(routers
        .iter()
        .any(|entry| entry.node.as_file() == Some(router.as_path())));

    let computed = graph.deps_of(
        &[NodeId::file(root.join("src/computed.ts"))],
        None,
        Some(&[EdgeKind::TrpcCall].into()),
    );
    assert!(computed.is_empty());
}

#[test]
fn trpc_router_globs_prefix_project_roots_and_skip_invalid_patterns() {
    assert_eq!(
        prefix_project_globs(Some("packages/api"), &["src/**/*.ts".into()]),
        vec!["packages/api/src/**/*.ts".to_string()]
    );
    assert_eq!(
        prefix_project_globs(Some("packages/api"), &["packages/api/src/**/*.ts".into()]),
        vec!["packages/api/src/**/*.ts".to_string()]
    );
    assert_eq!(
        prefix_project_globs(Some("."), &["src.ts".into()]),
        vec!["src.ts".to_string()]
    );
    assert_eq!(
        prefix_project_globs(Some("./packages/api"), &["./src/router.ts".into()]),
        vec!["packages/api/src/router.ts".to_string()]
    );
    assert_eq!(
        prefix_project_globs(
            Some("./packages/api"),
            &["./packages/api/src/router.ts".into()]
        ),
        vec!["packages/api/src/router.ts".to_string()]
    );
    assert_eq!(
        prefix_project_globs(Some(""), &["src.ts".into()]),
        vec!["src.ts".to_string()]
    );
    assert_eq!(
        prefix_project_globs(None, &["src.ts".into()]),
        vec!["src.ts".to_string()]
    );
    assert!(glob_has_root_prefix("packages/api", "packages/api"));
    assert!(glob_has_root_prefix("packages/api/src", "packages/api"));
    assert!(!glob_has_root_prefix("packages/apit", "packages/api"));
    assert!(compile_trpc_router_globset(&[]).is_none());
    assert!(compile_trpc_router_globset(&["[".into()]).is_none());
    assert!(compile_trpc_router_globset(&["[".into(), "src/**".into()]).is_some());
}

#[test]
fn collect_trpc_edges_skip_missing_config_facts_and_invalid_globs() {
    let files = GraphFiles::from_files(vec![PathBuf::from("/repo/src/router.ts")]);
    let interner = crate::codebase::analysis_session::PathInterner::new();
    let root = std::path::Path::new("/repo");
    assert!(collect_trpc_edges(root, &files, None, None, &interner).is_empty());

    let empty = GraphConfigOptions {
        trpc_routers: Vec::new(),
        ..GraphConfigOptions::default()
    };
    assert!(collect_trpc_edges(root, &files, None, Some(&empty), &interner).is_empty());

    let invalid = GraphConfigOptions {
        trpc_routers: vec!["[".into()],
        ..GraphConfigOptions::default()
    };
    assert!(collect_trpc_edges(root, &files, None, Some(&invalid), &interner).is_empty());

    let configured = GraphConfigOptions {
        trpc_routers: vec!["src/**".into()],
        ..GraphConfigOptions::default()
    };
    assert!(collect_trpc_edges(root, &files, None, Some(&configured), &interner).is_empty());
}

#[test]
fn trpc_fact_context_is_opt_in_even_when_routers_are_configured() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("trpc-basic"));
    let options = graph_config_options(&root).unwrap();
    assert!(trpc_facts_configured(&options));
    let enabled = ts_fact_context_from_options(
        &root,
        GraphBuildPlan {
            trpc: true,
            ..GraphBuildPlan::default()
        },
        Some(&options),
    );
    assert!(enabled.trpc_router_glob.is_some());
    let skipped = ts_fact_context_from_options(&root, GraphBuildPlan::default(), Some(&options));
    assert!(skipped.trpc_router_glob.is_none());
}

#[test]
fn unconfigured_trpc_does_not_change_import_edges() {
    let import_plan = GraphBuildPlan {
        imports: true,
        ..GraphBuildPlan::default()
    };
    let trpc_plan = GraphBuildPlan {
        imports: true,
        trpc: true,
        ..GraphBuildPlan::default()
    };
    let (root, import_graph) = trpc_graph("trpc-unconfigured", import_plan);
    let (_, trpc_graph) = trpc_graph("trpc-unconfigured", trpc_plan);
    let client = NodeId::file(root.join("src/client.ts"));
    let import_filter: std::collections::HashSet<EdgeKind> = [EdgeKind::Import].into();
    let left = import_graph.deps_of(std::slice::from_ref(&client), None, Some(&import_filter));
    let right = trpc_graph.deps_of(std::slice::from_ref(&client), None, Some(&import_filter));
    assert_eq!(left, right);

    let trpc_filter: std::collections::HashSet<EdgeKind> =
        [EdgeKind::TrpcCall, EdgeKind::TrpcProcedure].into();
    assert!(trpc_graph
        .deps_of(std::slice::from_ref(&client), None, Some(&trpc_filter))
        .is_empty());
}

#[test]
fn unfiltered_graph_omits_configured_trpc_edges() {
    let (root, graph) = trpc_graph("trpc-basic", GraphBuildPlan::all());
    let client = NodeId::file(root.join("src/client.ts"));
    let trpc_filter: std::collections::HashSet<EdgeKind> =
        [EdgeKind::TrpcCall, EdgeKind::TrpcProcedure].into();
    assert!(graph
        .deps_of(&[client], None, Some(&trpc_filter))
        .is_empty());
}
