use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn swift_edge_collector_covers_empty_config_branches() {
    let root = fixture("swift-test-plan");
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let all_files = GraphFiles::discover(&root).all;

    assert!(collect_swift_edges(&root, &tsconfig, &all_files, None).is_empty());

    let mut options = graph_config_options(&root).expect("swift fixture config should parse");
    options.swift_packages.clear();
    assert!(collect_swift_edges(&root, &tsconfig, &all_files, Some(&options)).is_empty());

    let options = graph_config_options(&root).expect("swift fixture config should parse");
    assert!(collect_swift_edges(&root, &tsconfig, &[], Some(&options)).is_empty());
}

#[test]
fn swift_edge_helpers_emit_import_reference_and_package_edges() {
    let source = p("Client/Sources/App/App.swift");
    let dependency = p("Client/Sources/Core/Core.swift");
    let mut facts = crate::codebase::swift::SwiftFactMap::default();
    facts.files.insert(
        source.clone(),
        crate::codebase::swift::SwiftFileFacts {
            path: source.clone(),
            target: Some("App".to_string()),
            imports: vec!["Core".to_string()],
            references: vec!["CoreClient".to_string()],
            ..Default::default()
        },
    );
    facts.files.insert(
        dependency.clone(),
        crate::codebase::swift::SwiftFileFacts {
            path: dependency.clone(),
            target: Some("Core".to_string()),
            declarations: vec!["CoreClient".to_string()],
            ..Default::default()
        },
    );
    facts.declarations.insert(
        "CoreClient".to_string(),
        BTreeSet::from([dependency.clone()]),
    );
    facts
        .files_by_target
        .insert("App".to_string(), BTreeSet::from([source]));
    facts
        .files_by_target
        .insert("Core".to_string(), BTreeSet::from([dependency]));
    facts
        .packages
        .push(crate::codebase::swift::SwiftPackageFacts {
            package_root: p("Client"),
            targets: BTreeMap::from([(
                "App".to_string(),
                crate::codebase::swift::SwiftTargetFacts {
                    name: "App".to_string(),
                    dependencies: vec!["Core".to_string()],
                    ..Default::default()
                },
            )]),
        });

    let mut edges = Vec::new();
    collect_swift_import_edges(&facts, &mut edges);
    collect_swift_reference_edges(&facts, &mut edges);
    collect_swift_package_edges(&facts, &mut edges);

    assert!(edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::SwiftImport));
    assert!(edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::SwiftReference));
    assert!(edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::SwiftPackageDependency));
}

#[test]
fn swift_http_edge_helper_covers_configured_route_lookup_without_matches() {
    let root = fixture("swift-test-plan");
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let all_files = GraphFiles::discover(&root).all;
    let options = graph_config_options(&root).expect("swift fixture config should parse");
    let swift_file = root.join("swift-clients/core/Sources/VouchaAPI/Endpoint.swift");
    let mut facts = crate::codebase::swift::SwiftFactMap::default();
    facts.files.insert(
        swift_file.clone(),
        crate::codebase::swift::SwiftFileFacts {
            path: swift_file,
            endpoint_paths: vec!["/api/v1/feeds/rss_feed_items/*".to_string()],
            ..Default::default()
        },
    );

    let mut edges = Vec::new();
    collect_swift_http_edges(
        &root, &tsconfig, &all_files, &options, None, &facts, &mut edges,
    );
    assert!(edges.iter().all(|(_, _, kind)| *kind == EdgeKind::HttpCall));
}

#[test]
fn swift_http_edges_include_backend_route_defs() {
    let root = fixture("swift-test-plan");
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let all_files = GraphFiles::discover(&root).all;
    let mut options = graph_config_options(&root).expect("swift fixture config should parse");
    options.route.backend_pattern = "backend/api/**/*.mts".to_string();
    options.route.backend_register_object = "app".to_string();
    let swift_file = root.join("swift-clients/core/Sources/VouchaAPI/Endpoint.swift");
    let mut facts = crate::codebase::swift::SwiftFactMap::default();
    facts.files.insert(
        swift_file.clone(),
        crate::codebase::swift::SwiftFileFacts {
            path: swift_file.clone(),
            endpoint_paths: vec!["/api/v1/feeds/rss_feed_items/*".to_string()],
            ..Default::default()
        },
    );

    let mut edges = Vec::new();
    collect_swift_http_edges(
        &root, &tsconfig, &all_files, &options, None, &facts, &mut edges,
    );

    assert!(edges.iter().all(|(_, _, kind)| *kind == EdgeKind::HttpCall));
}

#[test]
fn swift_package_edges_skip_targets_without_files() {
    let mut facts = crate::codebase::swift::SwiftFactMap::default();
    facts
        .packages
        .push(crate::codebase::swift::SwiftPackageFacts {
            package_root: p("Client"),
            targets: BTreeMap::from([(
                "Missing".to_string(),
                crate::codebase::swift::SwiftTargetFacts {
                    name: "Missing".to_string(),
                    dependencies: vec!["Core".to_string()],
                    ..Default::default()
                },
            )]),
        });

    let mut edges = Vec::new();
    collect_swift_package_edges(&facts, &mut edges);

    assert!(edges.is_empty());
}

#[test]
fn project_route_only_swift_http_edges_reuse_prepared_server_facts_once() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/project-server-routes-swift"),
    );
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let all_files = GraphFiles::discover(&root).all;
    let options = graph_config_options(&root).unwrap();
    let plan = GraphBuildPlan {
        swift: true,
        ..GraphBuildPlan::default()
    };
    let fact_plan = effective_ts_fact_plan(plan, Some(&options));
    assert!(fact_plan.server_routes);
    assert!(!fact_plan.backend_routes);
    let ts_facts = collect_ts_facts_with_context(
        &all_files,
        fact_plan,
        &ts_fact_context_from_options(&root, plan, Some(&options)),
    );
    let swift_facts =
        crate::codebase::swift::collect_swift_facts(&root, &all_files, &options.swift_packages);

    let standalone = collect_swift_edges_with_facts(
        &root,
        &tsconfig,
        &all_files,
        Some(&options),
        None,
        Some(&swift_facts),
    );
    let reused = collect_swift_edges_with_facts(
        &root,
        &tsconfig,
        &all_files,
        Some(&options),
        Some(&ts_facts),
        Some(&swift_facts),
    );
    let swift_file = root.join("swift-client/Sources/Client/Endpoint.swift");
    let admin_route = root.join("backend/api/admin-router.ts");

    assert_eq!(reused, standalone);
    assert!(reused.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::HttpCall
            && from.as_file() == Some(swift_file.as_path())
            && to.as_file() == Some(admin_route.as_path())
    }));

    crate::ast::begin_parse_count(&root);
    let graph = DepGraph::build_with_plan(&root, &tsconfig, plan).unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert!(graph
        .dependencies_of_node(&NodeId::File(swift_file))
        .is_some_and(|edges| edges.iter().any(|(to, kind)| {
            *kind == EdgeKind::HttpCall && to.as_file() == Some(admin_route.as_path())
        })));
    assert_eq!(counts.get(&root.join("backend/api/admin-router.ts")), Some(&1));
    assert_eq!(counts.get(&root.join("backend/api/users.ts")), Some(&1));
    assert!(
        counts.values().all(|count| *count == 1),
        "Swift graph TS sources must be parsed once: {counts:#?}"
    );
}
