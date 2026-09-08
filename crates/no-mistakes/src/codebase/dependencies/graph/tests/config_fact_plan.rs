use super::*;

#[test]
fn effective_fact_plan_skips_config_dependent_domains_without_required_config() {
    let requested = GraphBuildPlan {
        routes: true,
        queues: true,
        http: true,
        trpc: true,
        ..GraphBuildPlan::default()
    };
    assert!(effective_ts_fact_plan(requested, None).is_empty());

    let empty = crate::codebase::ts_resolver::normalize_path(&fixture("graph-empty-route-config"));
    let empty_options = graph_config_options(&empty).unwrap();
    assert!(effective_ts_fact_plan(requested, Some(&empty_options)).is_empty());

    let explicit =
        crate::codebase::ts_resolver::normalize_path(&fixture("graph-default-route-config"));
    let explicit_options = graph_config_options(&explicit).unwrap();
    let route_and_http = effective_ts_fact_plan(requested, Some(&explicit_options));
    assert!(route_and_http.route_refs);
    assert!(route_and_http.backend_routes);
    assert!(route_and_http.http_calls);
    assert!(!route_and_http.symbols);
    assert!(!route_and_http.queue_usage);
    assert!(!route_and_http.queue_factory);

    let queue_options = GraphConfigOptions {
        route: crate::codebase::config::RouteOptions::default(),
        queue: crate::codebase::config::QueueOptions {
            queue_pattern: "src/**/*.ts".to_string(),
            factory_specifier: "@app/queue".to_string(),
            factory_function: "createQueue".to_string(),
        },
        http_route: crate::codebase::config::HttpRouteOptions::default(),
        http_call: crate::codebase::config::HttpCallOptions::default(),
        project_route_globset: None,
        test_filter: None,
        rewrites: vec![],
        queue_project_factory_names: vec!["createQueue".to_string()],
        dotnet_projects: vec![],
        swift_packages: vec![],
        python_packages: vec![],
        go_modules: vec![],
        rust_packages: vec![],
        rails_apps: vec![],
        php_apps: vec![],
        php_framework: None,
        java_packages: vec![],
        kotlin_packages: vec![],
        elixir_apps: vec![],
        dart_packages: vec![],
        queue_enqueues: vec![],
        queue_workers: vec![],
        queue_cluster: None,
        queue_glob_clusters: HashMap::new(),
        trpc_routers: Vec::new(),
        terraform: Default::default(),
        ci: crate::config::v2::schema::CiConfig::default(),
    };
    let queue_only = effective_ts_fact_plan(
        GraphBuildPlan {
            queues: true,
            ..GraphBuildPlan::default()
        },
        Some(&queue_options),
    );
    assert!(queue_only.symbols);
    assert!(queue_only.queue_usage);
    assert!(queue_only.queue_factory);
    assert!(queue_only.queue_project);
}

#[test]
fn call_edges_request_only_binding_aware_facts() {
    let facts = GraphBuildPlan {
        calls: true,
        ..GraphBuildPlan::default()
    }
    .ts_fact_plan();
    assert!(facts.call_reachability);
    assert!(!facts.imports);
    assert!(facts.function_calls);
    assert!(facts.symbols);
}

#[test]
fn call_edges_project_prepared_binding_facts_deterministically() {
    use crate::codebase::dependencies::extract::{CallBinding, CallReachabilityFact};
    use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};

    let path = PathBuf::from("/repo/src/calls.ts");
    let mut facts = TsFactMap::new();
    facts.insert(
        path.clone(),
        TsFileFacts {
            call_reachability: vec![
                CallReachabilityFact {
                    caller: Some("run".to_string()),
                    callee: "setTimeout".to_string(),
                    binding: CallBinding::Global {
                        name: "setTimeout".to_string(),
                    },
                    line: 4,
                    invocation_kind: "direct",
                },
                CallReachabilityFact {
                    caller: Some("run".to_string()),
                    callee: "helper".to_string(),
                    binding: CallBinding::Local {
                        scope: "helper".to_string(),
                    },
                    line: 3,
                    invocation_kind: "direct",
                },
                // Dynamic calls stay available to a policy's unknown-call
                // handling, but never fabricate a graph target.
                CallReachabilityFact {
                    caller: Some("run".to_string()),
                    callee: "computed".to_string(),
                    binding: CallBinding::Unresolved {
                        display: "computed".to_string(),
                    },
                    line: 5,
                    invocation_kind: "unknown",
                },
            ],
            ..Default::default()
        },
    );
    let files = GraphFiles::from_files(vec![path.clone()]);
    let config = TsConfig::default();
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&config);
    let edges = collect_call_reachability_edges(
        files.indexable(),
        &facts,
        &resolver,
        &crate::codebase::workspaces::IndexedWorkspaceMap::default(),
        &files,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    assert_eq!(edges.len(), 2);
    assert!(edges.iter().all(|(_, _, kind)| *kind == EdgeKind::Call));
    assert!(edges.iter().any(|(_, target, _)| {
        target == &NodeId::symbol(&path, "helper")
    }));
    assert!(edges.iter().any(|(_, target, _)| {
        target == &NodeId::module("global:setTimeout")
    }));
}

fn call_graph_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/call-reachability")
            .join(name),
    )
}

fn build_call_graph(root: &Path, files: Vec<PathBuf>) -> DepGraph {
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json"))
        .expect("call graph fixture tsconfig should load");
    DepGraph::build_with_plan_files_config_and_facts(
        root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..GraphBuildPlan::default()
        },
        &GraphFiles::from_files(files),
        None,
        None,
    )
    .expect("call graph fixture should build")
}

#[test]
fn call_only_plan_follows_caller_through_barrel_to_source_symbol() {
    let root = call_graph_fixture("graph-barrel");
    let caller = root.join("caller.ts");
    let barrel = root.join("barrel.ts");
    let source = root.join("source.ts");
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json"))
        .expect("call graph fixture tsconfig should load");
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    assert_eq!(resolver.resolve("./barrel", &caller), Some(barrel.clone()));
    let graph = build_call_graph(&root, vec![caller.clone(), barrel.clone(), source.clone()]);

    let caller_symbol = NodeId::symbol(&caller, "run");
    let barrel_symbol = NodeId::symbol(&barrel, "createProgram");
    let direct = graph.dependencies_of_node(&caller_symbol);
    assert!(
        direct.is_some_and(|edges| edges.contains(&(barrel_symbol.clone(), EdgeKind::Call))),
        "direct caller edges: {direct:#?}"
    );
    assert!(direct.is_some_and(|edges| {
        edges.contains(&(NodeId::symbol(&source, "client/create"), EdgeKind::Call))
    }));
    assert!(direct.is_some_and(|edges| {
        edges.contains(&(NodeId::symbol(&source, "api/run"), EdgeKind::Call))
    }));
    let traversed = graph.deps_of(&[caller_symbol], None, None);
    assert!(traversed.iter().any(|entry| {
        entry.node == barrel_symbol && entry.depth == 1 && entry.via.contains(&EdgeKind::Call)
    }));
    assert!(
        traversed.iter().any(|entry| {
            entry.node == NodeId::symbol(&source, "createProgram")
                && entry.depth == 2
        }),
        "call-only traversal: {traversed:#?}"
    );

    let allowed = HashSet::from([EdgeKind::Call, EdgeKind::CallReexport]);
    let call_trace = graph.deps_of(&[NodeId::symbol(&caller, "run")], None, Some(&allowed));
    assert_eq!(call_trace.len(), 4, "call-specific traversal: {call_trace:#?}");
    assert!(call_trace
        .iter()
        .all(|entry| entry.node != NodeId::symbol(&source, "value")));
    let source_trace = call_trace
        .iter()
        .find(|entry| entry.node == NodeId::symbol(&source, "createProgram"))
        .expect("barrel traversal should reach the source declaration");
    assert_eq!(source_trace.depth, 2);
    assert_eq!(
        source_trace.via,
        vec![EdgeKind::Call, EdgeKind::CallReexport]
    );
    assert_eq!(
        graph
            .deps_of(
                &[NodeId::symbol(&caller, "run")],
                Some(1),
                Some(&allowed),
            )
            .len(),
        3
    );
}

#[test]
fn call_only_plan_resolves_workspace_package_exports() {
    let root = call_graph_fixture("workspace");
    let caller = root.join("packages/app/src/caller.ts");
    let root_manifest = root.join("package.json");
    let app_manifest = root.join("packages/app/package.json");
    let package_manifest = root.join("packages/lib/package.json");
    let target = root.join("packages/lib/src/index.ts");
    let graph = build_call_graph(
        &root,
        vec![caller.clone(), root_manifest, app_manifest, package_manifest, target.clone()],
    );

    assert!(graph
        .dependencies_of_node(&NodeId::symbol(&caller, "run"))
        .is_some_and(|edges| edges.contains(&(NodeId::symbol(&target, "createProgram"), EdgeKind::Call))));
}

#[test]
fn call_only_plan_keeps_external_named_namespace_and_star_identity() {
    let root = call_graph_fixture("graph-external");
    let caller = root.join("caller.ts");
    let barrel = root.join("barrel.ts");
    let graph = build_call_graph(&root, vec![caller.clone(), barrel]);
    let allowed = HashSet::from([EdgeKind::Call, EdgeKind::CallReexport]);
    let traversed = graph.deps_of(
        &[NodeId::symbol(&caller, "run")],
        None,
        Some(&allowed),
    );

    for module in ["pkg#fn", "pkg2#member", "pkg3#starFn"] {
        assert!(
            traversed
                .iter()
                .any(|entry| entry.node == NodeId::module(module)),
            "missing {module}: {traversed:#?}"
        );
    }
    assert!(traversed
        .iter()
        .all(|entry| entry.node != NodeId::module("pkg3#value")));
}

#[test]
fn call_only_plan_follows_star_and_namespace_reexports_without_value_reads() {
    let root = call_graph_fixture("graph-star");
    let caller = root.join("caller.ts");
    let barrel = root.join("barrel.ts");
    let source = root.join("source.ts");
    let graph = build_call_graph(&root, vec![caller.clone(), barrel.clone(), source.clone()]);
    let allowed = HashSet::from([EdgeKind::Call, EdgeKind::CallReexport]);

    let direct = graph.deps_of(
        &[NodeId::symbol(&caller, "run")],
        None,
        Some(&allowed),
    );
    assert!(direct.iter().any(|entry| {
        entry.node == NodeId::symbol(&source, "direct")
            && entry.depth == 2
            && entry.via == vec![EdgeKind::CallReexport]
    }));
    assert!(direct.iter().any(|entry| {
        entry.node == NodeId::symbol(&source, "namespaced")
            && entry.depth == 1
            && entry.via == vec![EdgeKind::Call]
    }));
    assert!(direct
        .iter()
        .all(|entry| entry.node != NodeId::symbol(&source, "value")));
}

#[test]
fn call_specific_traversal_terminates_cycles_with_shortest_traces() {
    let root = call_graph_fixture("cycle");
    let file = root.join("index.ts");
    let graph = build_call_graph(&root, vec![file.clone()]);

    let root_node = NodeId::symbol(&file, "start");
    let allowed = HashSet::from([EdgeKind::Call, EdgeKind::CallReexport]);
    let entries = graph.deps_of(std::slice::from_ref(&root_node), None, Some(&allowed));
    assert_eq!(entries.len(), 2, "cycle traversal: {entries:#?}");
    assert_eq!(entries[0].node, NodeId::symbol(&file, "first"));
    assert_eq!(entries[0].depth, 1);
    assert_eq!(entries[1].node, NodeId::symbol(&file, "second"));
    assert_eq!(entries[1].depth, 2);
}
