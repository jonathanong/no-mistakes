use super::*;

#[test]
fn swift_edge_helpers_emit_import_reference_and_package_edges() {
    let source = p("Client/Sources/App/App.swift");
    let dependency = p("Client/Sources/Core/Core.swift");
    let mut facts = crate::codebase::swift::SwiftFactMap::default();
    facts.files.insert(source.clone(), crate::codebase::swift::SwiftFileFacts {
        path: source.clone(), target: Some("App".to_string()), imports: vec!["Core".to_string()], references: vec!["CoreClient".to_string()], ..Default::default()
    });
    facts.files.insert(dependency.clone(), crate::codebase::swift::SwiftFileFacts {
        path: dependency.clone(), target: Some("Core".to_string()), declarations: vec!["CoreClient".to_string()], ..Default::default()
    });
    facts.declarations.insert("CoreClient".to_string(), BTreeSet::from([dependency.clone()]));
    facts.files_by_target.insert("App".to_string(), BTreeSet::from([source]));
    facts.files_by_target.insert("Core".to_string(), BTreeSet::from([dependency]));
    facts.packages.push(crate::codebase::swift::SwiftPackageFacts {
        package_root: p("Client"),
        targets: BTreeMap::from([
            ("App".to_string(), crate::codebase::swift::SwiftTargetFacts { name: "App".to_string(), dependencies: vec!["Core".to_string()], ..Default::default() }),
            ("Core".to_string(), crate::codebase::swift::SwiftTargetFacts { name: "Core".to_string(), ..Default::default() }),
        ]),
        ..Default::default()
    });

    let mut edges = Vec::new();
    let interner = crate::codebase::analysis_session::PathInterner::new();
    collect_swift_import_edges(&facts, &mut edges, &interner);
    collect_swift_reference_edges(&facts, &mut edges, &interner);
    collect_swift_package_edges(&facts, &mut edges, &interner);

    assert!(edges.iter().any(|(_, _, kind)| *kind == EdgeKind::SwiftImport));
    assert!(edges.iter().any(|(_, _, kind)| *kind == EdgeKind::SwiftReference));
    assert!(edges.iter().any(|(_, _, kind)| *kind == EdgeKind::SwiftPackageDependency));
}

#[test]
fn swift_import_edges_resolve_duplicate_target_names_only_in_declared_packages() {
    let app = p("Client/App/Sources/App/App.swift");
    let declared_core = p("Client/Core/Sources/Core/Core.swift");
    let unrelated_core = p("Other/Core/Sources/Core/Core.swift");
    let mut facts = crate::codebase::swift::SwiftFactMap::default();
    facts.files.insert(app.clone(), crate::codebase::swift::SwiftFileFacts {
        path: app.clone(), target: Some("App".to_string()), imports: vec!["Core".to_string()], ..Default::default()
    });
    for path in [&declared_core, &unrelated_core] {
        facts.files.insert(path.clone(), crate::codebase::swift::SwiftFileFacts {
            path: path.clone(), target: Some("Core".to_string()), ..Default::default()
        });
    }
    facts.files_by_target.insert("Core".to_string(), BTreeSet::from([declared_core.clone(), unrelated_core.clone()]));
    facts.packages.extend([
        crate::codebase::swift::SwiftPackageFacts {
            package_root: p("Client/App"),
            local_package_paths: vec!["../Core".to_string()],
            local_package_bindings: BTreeMap::from([("../Core".to_string(), "core".to_string())]),
            targets: BTreeMap::from([("App".to_string(), crate::codebase::swift::SwiftTargetFacts {
                name: "App".to_string(), dependencies: vec!["Core".to_string()], product_packages: BTreeMap::from([("Core".to_string(), "core".to_string())]), ..Default::default()
            })]),
            ..Default::default()
        },
        crate::codebase::swift::SwiftPackageFacts {
            package_root: p("Client/Core"),
            products: BTreeMap::from([("Core".to_string(), vec!["Core".to_string()])]),
            targets: BTreeMap::from([("Core".to_string(), crate::codebase::swift::SwiftTargetFacts { name: "Core".to_string(), ..Default::default() })]),
            ..Default::default()
        },
        crate::codebase::swift::SwiftPackageFacts { package_root: p("Other/Core"), ..Default::default() },
    ]);

    let mut edges = Vec::new();
    collect_swift_import_edges(&facts, &mut edges, &crate::codebase::analysis_session::PathInterner::new());

    assert!(edges.contains(&(NodeId::file(app.clone()), NodeId::file(declared_core), EdgeKind::SwiftImport)));
    assert!(!edges.contains(&(NodeId::file(app), NodeId::file(unrelated_core), EdgeKind::SwiftImport)));
}

#[test]
fn swift_import_edges_keep_same_package_targets_for_unowned_sources() {
    let generated = p("Client/Generated/Runner.swift");
    let package_core = p("Client/Sources/Core/Core.swift");
    let unrelated_core = p("Other/Sources/Core/Core.swift");
    let mut facts = crate::codebase::swift::SwiftFactMap::default();
    facts.files.insert(
        generated.clone(),
        crate::codebase::swift::SwiftFileFacts {
            path: generated.clone(),
            imports: vec!["Core".to_string()],
            ..Default::default()
        },
    );
    for path in [&package_core, &unrelated_core] {
        facts.files.insert(
            path.clone(),
            crate::codebase::swift::SwiftFileFacts {
                path: path.clone(),
                target: Some("Core".to_string()),
                ..Default::default()
            },
        );
    }
    facts.files_by_target.insert(
        "Core".to_string(),
        BTreeSet::from([package_core.clone(), unrelated_core.clone()]),
    );
    facts.packages.extend([
        crate::codebase::swift::SwiftPackageFacts {
            package_root: p("Client"),
            targets: BTreeMap::from([(
                "Core".to_string(),
                crate::codebase::swift::SwiftTargetFacts {
                    name: "Core".to_string(),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        },
        crate::codebase::swift::SwiftPackageFacts {
            package_root: p("Other"),
            targets: BTreeMap::from([(
                "Core".to_string(),
                crate::codebase::swift::SwiftTargetFacts {
                    name: "Core".to_string(),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        },
    ]);

    let mut edges = Vec::new();
    collect_swift_import_edges(
        &facts,
        &mut edges,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    assert!(edges.contains(&(
        NodeId::file(generated.clone()),
        NodeId::file(package_core),
        EdgeKind::SwiftImport,
    )));
    assert!(!edges.contains(&(
        NodeId::file(generated), NodeId::file(unrelated_core), EdgeKind::SwiftImport,)));
}

#[test]
fn swift_import_edges_keep_custom_and_executable_target_imports() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/swift-target-ownership/fixture");
    let core = root.join("Sources/Core/Core.swift");
    let runner = root.join("Tools/Runner/Runner.swift");
    let custom_test = root.join("Checks/Integration/CustomTests.swift");
    let plugin = root.join("Tooling/Plugin/Plugin.swift");
    let facts = crate::codebase::swift::collect_swift_facts(
        &root,
        &[core.clone(), runner.clone(), custom_test.clone(), plugin.clone()],
        &[".".to_string()],
    );

    let mut edges = Vec::new();
    collect_swift_import_edges(
        &facts,
        &mut edges,
        &crate::codebase::analysis_session::PathInterner::new(),
    );

    for source in [runner, custom_test, plugin] {
        assert!(edges.contains(&(
            NodeId::file(source),
            NodeId::file(core.clone()),
            EdgeKind::SwiftImport,
        )));
    }
}
