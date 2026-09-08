fn call_fixture_graph() -> (PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("call-traversal"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            imports: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    (root, graph)
}

fn symbol(path: &Path, name: &str) -> NodeId {
    NodeId::symbol(path, name)
}

/// Traversal nodes carry an opaque callable identity so same-named sibling
/// declarations cannot collapse. These catalog assertions intentionally
/// check the stable public file/symbol projection instead.
fn has_symbol(node: &NodeId, path: &Path, name: &str) -> bool {
    matches!(
        node,
        NodeId::Symbol { file, symbol, .. }
            if file.as_ref() == path && symbol.as_ref() == name
    )
}

fn call_resolution_inputs<'a>(
    root: &'a Path,
    tsconfig: &'a TsConfig,
    graph_files: &'a GraphFiles,
) -> GraphEdgeBuildInputs<'a> {
    GraphEdgeBuildInputs {
        root,
        tsconfig,
        tsconfig_catalog: None,
        plan: GraphBuildPlan::default(),
        workspace: None,
        graph_files,
        config_options: None,
        playwright_settings: &[],
        config_path: None,
        dotnet_facts: None,
        swift_facts: None,
        import_resolution_cache: None,
        visible_paths: None,
        workflow_documents: None,
        interner: Arc::new(PathInterner::new()),
    }
}

#[path = "call_traversal/export_resolution.rs"]
mod export_resolution;

#[test]
fn call_traversal_has_direct_file_and_transitive_depth_boundaries() {
    let (root, graph) = call_fixture_graph();
    let entry = root.join("src/entry.mts");
    let entry_root = symbol(&entry, "entry");

    let direct = graph.call_traces(
        std::slice::from_ref(&entry_root),
        CallTraversal::Direct,
        None,
    );
    let direct_targets = direct.iter().map(|trace| &trace.target).collect::<Vec<_>>();
    assert_eq!(direct.len(), 3, "entry has three direct calls");
    assert!(direct_targets.iter().any(|node| has_symbol(
        node,
        &root.join("src/diamond-left.mts"),
        "diamondLeft"
    )));
    assert!(direct_targets.iter().any(|node| has_symbol(
        node,
        &root.join("src/diamond-right.mts"),
        "diamondRight"
    )));
    assert!(direct_targets.iter().any(|node| has_symbol(
        node,
        &root.join("src/cycle-a.mts"),
        "cycleA"
    )));

    assert!(graph
        .call_traces(
            std::slice::from_ref(&entry_root),
            CallTraversal::Transitive,
            Some(0)
        )
        .is_empty());
    let depth_one = graph.call_traces(
        std::slice::from_ref(&entry_root),
        CallTraversal::Transitive,
        Some(1),
    );
    assert_eq!(depth_one, direct, "maxDepth=1 is the direct boundary");
    let depth_two = graph.call_traces(&[entry_root], CallTraversal::Transitive, Some(2));
    assert!(depth_two.iter().any(|trace| {
        has_symbol(
            &trace.target,
            &root.join("src/diamond-shared.mts"),
            "shared",
        )
    }));
    assert!(depth_two.iter().any(|trace| has_symbol(
        &trace.target,
        &root.join("src/cycle-b.mts"),
        "cycleB"
    )));

    // A file traversal is deliberately a same-source-file layer, even when
    // the root function calls imported functions.
    assert!(graph
        .call_traces(
            &[symbol(&root.join("src/local.mts"), "first")],
            CallTraversal::File,
            None,
        )
        .iter()
        .all(|trace| trace.target.as_file() == Some(root.join("src/local.mts").as_path())));
    assert!(graph
        .call_traces(&[symbol(&entry, "entry")], CallTraversal::File, None)
        .is_empty());
}

#[test]
fn class_member_membership_does_not_create_call_edges() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    let method = root.join("src/method.mts");
    let calls = graph.deps_of(
        &[symbol(&method, "Loader")],
        None,
        Some(&[EdgeKind::Call].into()),
    );

    assert!(
        calls.is_empty(),
        "class membership is not invocation evidence"
    );
}

#[test]
fn call_traversal_uses_deterministic_shortest_diamond_paths_and_terminates_cycles() {
    let (root, graph) = call_fixture_graph();
    let entry = root.join("src/entry.mts");
    let traces = graph.call_traces(&[symbol(&entry, "entry")], CallTraversal::Transitive, None);
    let shared_trace = traces
        .iter()
        .find(|trace| {
            has_symbol(
                &trace.target,
                &root.join("src/diamond-shared.mts"),
                "shared",
            )
        })
        .expect("diamond target is reachable");
    assert_eq!(shared_trace.nodes.len(), 3);
    assert!(has_symbol(
        &shared_trace.nodes[1],
        &root.join("src/diamond-left.mts"),
        "diamondLeft"
    ));
    assert_eq!(
        traces
            .iter()
            .filter(|trace| has_symbol(
                &trace.target,
                &root.join("src/diamond-shared.mts"),
                "shared"
            ))
            .count(),
        1,
        "diamond target has one shortest trace"
    );

    for name in ["cycleA", "cycleB", "cycleC"] {
        let path = root.join("src").join(format!(
            "cycle-{}.mts",
            name.trim_start_matches("cycle").to_ascii_lowercase()
        ));
        assert!(traces
            .iter()
            .any(|trace| has_symbol(&trace.target, &path, name)));
    }
    assert!(
        traces.len() < 20,
        "multi-node cycle must not expand forever"
    );
}

#[test]
fn call_roots_are_pure_and_retain_leaf_and_global_only_callables() {
    let (root, graph) = call_fixture_graph();
    let entry = root.join("src/entry.mts");
    let roots = graph.expand_call_roots(&[CallRoot::File(entry.clone())]);
    assert!(roots.contains(&NodeId::file(&entry)));
    assert!(roots.iter().all(|node| {
        node.as_file() == Some(entry.as_path())
            || matches!(node, NodeId::Symbol { file, .. } if file.as_ref() == entry.as_path())
    }));
    assert!(!roots.contains(&symbol(&root.join("src/cycle-a.mts"), "cycleA")));

    let global_file = root.join("src/global-only.mts");
    assert!(graph
        .expand_call_roots(&[CallRoot::File(global_file.clone())])
        .iter()
        .any(|node| has_symbol(node, &global_file, "globalOnly")));
    assert!(graph
        .expand_call_roots(&[CallRoot::Module(global_file.clone())])
        .contains(&NodeId::file(&global_file)));
    assert!(graph
        .expand_call_roots(&[CallRoot::Function {
            file: global_file.clone(),
            symbol: "notDefined".to_string(),
        }])
        .is_empty());

    let unknown_file = root.join("src/unknown.mts");
    let unknown_root = symbol(&unknown_file, "unknown");
    assert!(graph
        .call_traces(&[unknown_root], CallTraversal::Transitive, None)
        .is_empty());
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == unknown_file
            && site.source_callee == "globalThis.setTimeout"
            && matches!(site.target, ResolvedCallTarget::Unknown)
    }));
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == unknown_file
            && site.source_callee == "<unknown>"
            && matches!(site.target, ResolvedCallTarget::Unknown)
    }));
}

#[test]
fn exported_function_roots_resolve_renamed_defaults_and_barrels() {
    let (root, graph) = call_fixture_graph();

    for (file, symbol, target_file, target) in [
        (
            root.join("src/exported-local-aliases.mts"),
            "exportedAlias",
            root.join("src/alias-target.mts"),
            "importedTarget",
        ),
        (
            root.join("src/exported-local-aliases.mts"),
            "default",
            root.join("src/alias-target.mts"),
            "defaultTarget",
        ),
        (
            root.join("src/reexport-default.mts"),
            "default",
            root.join("src/alias-target.mts"),
            "defaultTarget",
        ),
        (
            root.join("src/star-cycle-a.mts"),
            "cycle",
            root.join("src/star-cycle-provider.mts"),
            "cycle",
        ),
        (
            root.join("src/star-cycle-b.mts"),
            "cycle",
            root.join("src/star-cycle-provider.mts"),
            "cycle",
        ),
        (
            root.join("src/unreferenced-export.mts"),
            "public",
            root.join("src/unreferenced-export.mts"),
            "uncalledTarget",
        ),
    ] {
        let roots = graph.expand_call_roots(&[CallRoot::Function {
            file,
            symbol: symbol.to_string(),
        }]);
        assert_eq!(roots.len(), 1, "{symbol} should resolve to one root");
        assert!(has_symbol(&roots[0], &target_file, target), "{roots:#?}");
    }
}

#[test]
fn exported_function_roots_follow_named_reexport_barrels() {
    let (root, graph) = call_fixture_graph();
    let barrel = root.join("src/exported-local-aliases.mts");
    let facts = collect_ts_facts(
        std::slice::from_ref(&barrel),
        TsFactPlan {
            imports: true,
            function_calls: true,
            ..TsFactPlan::default()
        },
    );
    let bindings = &facts.get(&barrel).unwrap().exported_bindings;
    let binding = bindings
        .iter()
        .find(|binding| binding.exported == "reexportedTarget")
        .unwrap_or_else(|| panic!("named re-export must be extracted: {bindings:#?}"));
    assert_eq!(binding.local, "public");
    assert_eq!(binding.specifier.as_deref(), Some("./unreferenced-export.mts"));
    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: barrel,
        symbol: "reexportedTarget".to_string(),
    }]);

    assert_eq!(
        roots.len(),
        1,
        "named re-export should resolve to one root; exports={:?}",
        graph
            .callable_export_resolutions
            .keys()
            .filter(|(file, _)| file == &root.join("src/exported-local-aliases.mts"))
            .collect::<Vec<_>>()
    );
    assert!(has_symbol(
        &roots[0],
        &root.join("src/unreferenced-export.mts"),
        "uncalledTarget"
    ));
}

#[test]
fn call_resolution_follows_immutable_aliases_and_reexported_defaults_only() {
    let (root, graph) = call_fixture_graph();
    let aliases = root.join("src/aliases.mts");
    let roots = graph.expand_call_roots(&[CallRoot::File(aliases.clone())]);
    let traces = graph.call_traces(&roots, CallTraversal::Direct, None);

    assert!(graph.call_traces(&[NodeId::file(&aliases)], CallTraversal::Direct, None).iter().any(
        |trace| matches!(&trace.target, NodeId::Symbol { symbol, .. } if symbol.starts_with("<anonymous:")),
    ));

    assert_eq!(
        traces
            .iter()
            .filter(|trace| has_symbol(
                &trace.target,
                &root.join("src/alias-target.mts"),
                "importedTarget"
            ))
            .count(),
        3,
        "module, anonymous callback, and nested function aliases retain canonical call edges"
    );
    for callee in ["first", "second"] {
        assert!(
            graph.resolved_call_sites().iter().any(|site| {
                site.file == aliases
                    && site.source_callee == callee
                    && matches!(
                        &site.target,
                        ResolvedCallTarget::ModuleExport {
                            repository_target: Some((file, scope)),
                            ..
                        } if file == &root.join("src/alias-target.mts") && scope == "importedTarget"
                    )
            }),
            "{callee} must retain its imported target before graph edge deduplication"
        );
    }
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == aliases
            && site.source_callee == "moduleAlias"
            && matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    repository_target: Some((file, scope)),
                    ..
                } if file == &root.join("src/alias-target.mts") && scope == "importedTarget"
            )
    }));
    for callee in ["cycle", "cycleFromB"] {
        assert!(
            graph.resolved_call_sites().iter().any(|site| {
                site.file == aliases
                    && site.source_callee == callee
                    && matches!(
                        &site.target,
                        ResolvedCallTarget::ModuleExport {
                            repository_target: Some((file, scope)),
                            ..
                        } if file == &root.join("src/star-cycle-provider.mts") && scope == "cycle"
                    )
            }),
            "{callee} must resolve through the cyclic barrel to its concrete provider"
        );
    }
    for (callee, expected_export, expected_scope) in [
        ("targets.importedTarget", "importedTarget", "importedTarget"),
        ("targets.default", "default", "defaultTarget"),
    ] {
        let matching_sites = graph
            .resolved_call_sites()
            .iter()
            .filter(|site| site.file == aliases && site.source_callee == callee)
            .collect::<Vec<_>>();
        assert!(
            matching_sites.iter().any(|site| {
                matches!(
                    &site.target,
                    ResolvedCallTarget::ModuleExport {
                        specifier,
                        export_path,
                        repository_target: Some((file, scope)),
                    } if specifier == "./alias-target.mts"
                        && export_path == expected_export
                        && file == &root.join("src/alias-target.mts")
                        && scope == expected_scope
                )
            }),
            "a namespace member must retain its concrete export target: {callee}: {matching_sites:#?}"
        );
    }
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == aliases
            && site.source_callee == "remote"
            && matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    specifier,
                    export_path,
                    repository_target: None,
                } if specifier == "unresolved-runtime-library" && export_path == "remote"
            )
    }));
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == aliases
            && site.source_callee == "importedExternal"
            && matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    specifier,
                    export_path,
                    repository_target: None,
                } if specifier == "vitest" && export_path == "mock"
            )
    }));
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == aliases
            && site.source_callee == "second"
            && matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    repository_target: Some((file, scope)),
                    ..
                } if file == &root.join("src/alias-target.mts") && scope == "importedTarget"
            )
    }));
    assert!(traces.iter().any(|trace| has_symbol(
        &trace.target,
        &root.join("src/alias-target.mts"),
        "defaultTarget"
    )));
    assert!(
        !traces.iter().any(|trace| {
            has_symbol(
                &trace.target,
                &root.join("src/mixed-star-callable.mts"),
                "collision",
            )
        }),
        "a non-callable export-star collision must not select the callable branch"
    );
    assert!(
        !traces.iter().any(|trace| {
            has_symbol(
                &trace.target,
                &root.join("src/mixed-star-callable.mts"),
                "externalCollision",
            )
        }),
        "an unresolved export-star candidate must not select the visible callable branch"
    );
    for (callee, expected_scope) in [
        ("exportedAlias", "importedTarget"),
        ("defaultAlias", "defaultTarget"),
    ] {
        assert!(
            graph.resolved_call_sites().iter().any(|site| {
                site.file == aliases
                    && site.source_callee == callee
                    && matches!(
                        &site.target,
                        ResolvedCallTarget::ModuleExport {
                            repository_target: Some((file, scope)),
                            ..
                        } if file == &root.join("src/alias-target.mts") && scope == expected_scope
                    )
            }),
            "{callee} must follow an exported immutable local alias"
        );
    }
    assert!(
        graph.resolved_call_sites().iter().any(|site| {
            site.file == aliases
                && site.source_callee == "localExportAlias"
                && matches!(
                    &site.target,
                    ResolvedCallTarget::ModuleExport {
                        repository_target: Some((file, scope)),
                        ..
                    } if file == &root.join("src/exported-local-aliases.mts") && scope == "localTarget"
                )
        }),
        "an exported local alias must retain its local callable target"
    );
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == aliases
            && site.source_callee == "window.setTimeout"
            && matches!(site.target, ResolvedCallTarget::Global { .. })
    }));
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == aliases
            && site.source_callee == "externalMock"
            && matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    specifier,
                    export_path,
                    repository_target: None,
                } if specifier == "vitest" && export_path == "mock"
            )
    }));
    let same_line_calls = graph
        .resolved_call_sites()
        .iter()
        .filter(|site| {
            site.file == aliases
                && matches!(site.source_callee.as_str(), "setTimeout" | "clearTimeout")
        })
        .collect::<Vec<_>>();
    assert_eq!(same_line_calls.len(), 2);
    assert!(
        same_line_calls[0].offset < same_line_calls[1].offset,
        "same-line call sites retain ordered source offsets"
    );
    for callee in [
        "defaultThroughStar",
        "ambiguous",
        "mutable",
        "mutableDeclaration",
        "cycleA",
        "collision",
        "externalCollision",
        "window.setTimeout",
        "targets.deep.member",
        "propertyImport.call",
        "hidden",
        "missingReexport",
        "absentThroughStar",
    ] {
        assert!(
            graph.resolved_call_sites().iter().any(|site| {
                site.file == aliases
                    && site.source_callee == callee
                    && matches!(site.target, ResolvedCallTarget::Unknown)
            }),
            "{callee} must remain unknown rather than guessed"
        );
    }
}

#[test]
fn call_resolution_walks_lexical_parent_scopes_without_guessing_properties() {
    let (root, graph) = call_fixture_graph();
    let local = root.join("src/local.mts");
    let outer = symbol(&local, "outer");
    let inner = symbol(&local, "outer/inner");
    let direct = graph.call_traces(std::slice::from_ref(&inner), CallTraversal::Direct, None);
    assert!(
        direct
            .iter()
            .map(|trace| &trace.target)
            .all(|target| has_symbol(target, &local, "outer/outerOnly"))
            && direct.len() == 1,
        "a nested function can call a callable declared by its lexical parent"
    );
    assert!(
        graph
            .call_traces(&[outer], CallTraversal::Transitive, None)
            .iter()
            .any(|trace| has_symbol(&trace.target, &local, "outer/outerOnly")),
        "the parent-local edge participates in ordinary traversal"
    );
}
