use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[path = "swift_edges/package_and_imports.rs"]
mod package_and_imports;

#[test]
fn swift_shorthand_product_dependency_resolves_imports_without_symbol_references() {
    let app = p("Client/App/Sources/App/App.swift");
    let declared_core = p("Client/Core/Sources/CoreModule/Core.swift");
    let unrelated_core = p("Other/Core/Sources/CoreModule/Core.swift");
    let mut facts = crate::codebase::swift::SwiftFactMap::default();
    facts.files.insert(
        app.clone(),
        crate::codebase::swift::SwiftFileFacts {
            path: app.clone(),
            target: Some("App".to_string()),
            imports: vec!["CoreModule".to_string()],
            ..Default::default()
        },
    );
    for path in [&declared_core, &unrelated_core] {
        facts.files.insert(
            path.clone(),
            crate::codebase::swift::SwiftFileFacts {
                path: path.clone(),
                target: Some("CoreModule".to_string()),
                ..Default::default()
            },
        );
    }
    facts.files_by_target.insert(
        "CoreModule".to_string(),
        BTreeSet::from([declared_core.clone(), unrelated_core.clone()]),
    );
    facts.packages.extend([
        crate::codebase::swift::SwiftPackageFacts {
            package_root: p("Client/App"),
            local_package_paths: vec!["../Core".to_string()],
            local_package_bindings: BTreeMap::from([("../Core".to_string(), "core".to_string())]),
            products: BTreeMap::new(),
            targets: BTreeMap::from([(
                "App".to_string(),
                crate::codebase::swift::SwiftTargetFacts {
                    name: "App".to_string(),
                    dependencies: vec!["CoreProduct".to_string()],
                    ..Default::default()
                },
            )]),
        },
        crate::codebase::swift::SwiftPackageFacts {
            package_root: p("Client/Core"),
            products: BTreeMap::from([("CoreProduct".to_string(), vec!["CoreModule".to_string()])]),
            targets: BTreeMap::from([(
                "CoreModule".to_string(),
                crate::codebase::swift::SwiftTargetFacts {
                    name: "CoreModule".to_string(),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        },
        crate::codebase::swift::SwiftPackageFacts {
            package_root: p("Other/Core"),
            products: BTreeMap::from([("CoreProduct".to_string(), vec!["CoreModule".to_string()])]),
            ..Default::default()
        },
    ]);

    let mut edges = Vec::new();
    let interner = crate::codebase::analysis_session::PathInterner::new();
    collect_swift_import_edges(&facts, &mut edges, &interner);
    collect_swift_package_edges(&facts, &mut edges, &interner);

    for kind in [EdgeKind::SwiftImport, EdgeKind::SwiftPackageDependency] {
        assert!(edges.contains(&(
            NodeId::file(app.clone()),
            NodeId::file(declared_core.clone()),
            kind,
        )));
        assert!(!edges.contains(&(
            NodeId::file(app.clone()),
            NodeId::file(unrelated_core.clone()),
            kind,
        )));
    }
}

#[test]
fn swift_manifest_edges_connect_package_sources_and_resolved_pins() {
    let package_root = p("Client");
    let source = package_root.join("Sources/App/App.swift");
    let test = package_root.join("Tests/AppTests/AppTests.swift");
    let manifest = package_root.join("Package.swift");
    let resolved = package_root.join("Package.resolved");
    let mut facts = crate::codebase::swift::SwiftFactMap::default();
    for path in [&source, &test] {
        facts.files.insert(
            path.clone(),
            crate::codebase::swift::SwiftFileFacts {
                path: path.clone(),
                ..Default::default()
            },
        );
    }
    facts
        .packages
        .push(crate::codebase::swift::SwiftPackageFacts {
            package_root: package_root.clone(),
            ..Default::default()
        });

    let mut edges = Vec::new();
    collect_swift_manifest_edges(
        &facts,
        &[manifest.clone(), resolved.clone()],
        &mut edges,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    assert!(edges.contains(&(
        NodeId::file(source),
        NodeId::file(manifest.clone()),
        EdgeKind::SwiftPackageDependency
    )));
    assert!(edges.contains(&(
        NodeId::file(test),
        NodeId::file(manifest.clone()),
        EdgeKind::SwiftPackageDependency
    )));
    assert!(edges.contains(&(
        NodeId::file(manifest),
        NodeId::file(resolved),
        EdgeKind::SwiftPackageDependency
    )));
}

#[test]
fn swift_manifest_edges_normalize_local_package_paths_without_source_imports() {
    let app_root = p("Client/App");
    let source = app_root.join("Sources/App/App.swift");
    let app_manifest = app_root.join("Package.swift");
    let core_manifest = p("Client/Core/Package.swift");
    let mut facts = crate::codebase::swift::SwiftFactMap::default();
    facts.files.insert(
        source.clone(),
        crate::codebase::swift::SwiftFileFacts {
            path: source.clone(),
            ..Default::default()
        },
    );
    facts
        .packages
        .push(crate::codebase::swift::SwiftPackageFacts {
            package_root: app_root,
            local_package_paths: vec!["../Core".to_string()],
            ..Default::default()
        });

    let mut edges = Vec::new();
    collect_swift_manifest_edges(
        &facts,
        &[app_manifest, core_manifest.clone()],
        &mut edges,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    assert!(edges.contains(&(
        NodeId::file(source),
        NodeId::file(core_manifest),
        EdgeKind::SwiftPackageDependency,
    )));
}

#[test]
fn swift_manifest_edges_skip_absent_own_and_local_dependency_manifests() {
    let app_root = p("Client/App");
    let source = app_root.join("Sources/App/App.swift");
    let manifest = app_root.join("Package.swift");
    let local_manifest = p("Client/Core/Package.swift");
    let mut facts = crate::codebase::swift::SwiftFactMap::default();
    facts.files.insert(
        source.clone(),
        crate::codebase::swift::SwiftFileFacts {
            path: source.clone(),
            ..Default::default()
        },
    );
    facts
        .packages
        .push(crate::codebase::swift::SwiftPackageFacts {
            package_root: app_root,
            local_package_paths: vec!["../Core".to_string()],
            ..Default::default()
        });
    let mut edges = Vec::new();
    let interner = crate::codebase::analysis_session::PathInterner::new();

    collect_swift_manifest_edges(&facts, &[], &mut edges, &interner);
    assert!(edges.is_empty());

    collect_swift_manifest_edges(&facts, std::slice::from_ref(&manifest), &mut edges, &interner);

    assert_eq!(
        edges,
        vec![(
            NodeId::file(source),
            NodeId::file(manifest),
            EdgeKind::SwiftPackageDependency,
        )]
    );
    assert!(!edges.iter().any(|(_, target, _)| {
        target.as_file() == Some(local_manifest.as_path())
    }));
}

#[test]
fn swift_edges_assign_nested_files_only_to_their_deepest_package() {
    let parent_root = p("Client");
    let nested_root = p("Client/Vendor/Core");
    let parent_source = parent_root.join("Sources/Shared/Shared.swift");
    let parent_dependency = parent_root.join("Sources/Core/Core.swift");
    let nested_source = nested_root.join("Sources/Shared/Shared.swift");
    let parent_manifest = parent_root.join("Package.swift");
    let nested_manifest = nested_root.join("Package.swift");
    let mut facts = crate::codebase::swift::SwiftFactMap::default();
    for (path, target) in [
        (&parent_source, "Shared"),
        (&parent_dependency, "Core"),
        (&nested_source, "Shared"),
    ] {
        facts.files.insert(
            path.clone(),
            crate::codebase::swift::SwiftFileFacts {
                path: path.clone(),
                target: Some(target.to_string()),
                ..Default::default()
            },
        );
    }
    facts.packages.extend([
        crate::codebase::swift::SwiftPackageFacts {
            package_root: parent_root,
            targets: BTreeMap::from([
                (
                    "Shared".to_string(),
                    crate::codebase::swift::SwiftTargetFacts {
                        name: "Shared".to_string(),
                        dependencies: vec!["Core".to_string()],
                        ..Default::default()
                    },
                ),
                (
                    "Core".to_string(),
                    crate::codebase::swift::SwiftTargetFacts {
                        name: "Core".to_string(),
                        ..Default::default()
                    },
                ),
            ]),
            ..Default::default()
        },
        crate::codebase::swift::SwiftPackageFacts {
            package_root: nested_root,
            targets: BTreeMap::from([(
                "Shared".to_string(),
                crate::codebase::swift::SwiftTargetFacts {
                    name: "Shared".to_string(),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        },
    ]);

    let mut edges = Vec::new();
    let interner = crate::codebase::analysis_session::PathInterner::new();
    collect_swift_package_edges(&facts, &mut edges, &interner);
    collect_swift_manifest_edges(
        &facts,
        &[parent_manifest.clone(), nested_manifest],
        &mut edges,
        &interner,
    );

    assert!(edges.contains(&(
        NodeId::file(parent_source),
        NodeId::file(parent_dependency.clone()),
        EdgeKind::SwiftPackageDependency,
    )));
    assert!(!edges.contains(&(
        NodeId::file(nested_source.clone()),
        NodeId::file(parent_dependency),
        EdgeKind::SwiftPackageDependency,
    )));
    assert!(!edges.contains(&(
        NodeId::file(nested_source),
        NodeId::file(parent_manifest),
        EdgeKind::SwiftPackageDependency,
    )));
}

#[test]
fn swift_http_edge_helper_covers_configured_route_lookup_without_matches() {
    let root = fixture("swift-test-plan");
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let all_files = GraphFiles::discover(&root).all().to_vec();
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
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();

    let mut edges = Vec::new();
    collect_swift_http_edges(
        SwiftRouteDefInputs {
            root: &root,
            tsconfig: &tsconfig,
            tsconfig_catalog: None,
            all_files: &all_files,
            config_options: &options,
            ts_facts: None,
            session: &session,
        },
        &facts,
        &mut edges,
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(edges.iter().all(|(_, _, kind)| *kind == EdgeKind::HttpCall));
}

#[test]
fn swift_http_edges_include_backend_route_defs() {
    let root = fixture("swift-test-plan");
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let all_files = GraphFiles::discover(&root).all().to_vec();
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
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();

    let mut edges = Vec::new();
    collect_swift_http_edges(
        SwiftRouteDefInputs {
            root: &root,
            tsconfig: &tsconfig,
            tsconfig_catalog: None,
            all_files: &all_files,
            config_options: &options,
            ts_facts: None,
            session: &session,
        },
        &facts,
        &mut edges,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    assert!(edges.iter().all(|(_, _, kind)| *kind == EdgeKind::HttpCall));
}

#[path = "swift_edges/prepared_routes.rs"]
mod prepared_routes;
#[path = "swift_edges/reference_scoping.rs"]
mod reference_scoping;
