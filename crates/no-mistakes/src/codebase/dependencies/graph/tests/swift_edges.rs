use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn swift_edge_collector_covers_empty_config_branches() {
    let root = fixture("swift-test-plan");
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
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
    facts.packages.push(crate::codebase::swift::SwiftPackageFacts {
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

    assert!(edges.iter().any(|(_, _, kind)| *kind == EdgeKind::SwiftImport));
    assert!(edges.iter().any(|(_, _, kind)| *kind == EdgeKind::SwiftReference));
    assert!(edges
        .iter()
        .any(|(_, _, kind)| *kind == EdgeKind::SwiftPackageDependency));
}

#[test]
fn swift_http_edge_helper_covers_configured_route_lookup_without_matches() {
    let root = fixture("swift-test-plan");
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
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
    collect_swift_http_edges(&root, &tsconfig, &all_files, &options, &facts, &mut edges);
    assert!(edges.iter().all(|(_, _, kind)| *kind == EdgeKind::HttpCall));
}
