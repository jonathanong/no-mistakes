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

#[test]
fn call_traversal_has_direct_file_and_transitive_depth_boundaries() {
    let (root, graph) = call_fixture_graph();
    let entry = root.join("src/entry.mts");
    let entry_root = symbol(&entry, "entry");

    let direct = graph.call_traces(&[entry_root.clone()], CallTraversal::Direct, None);
    let direct_targets = direct.iter().map(|trace| &trace.target).collect::<Vec<_>>();
    assert_eq!(direct.len(), 3, "entry has three direct calls");
    assert!(direct_targets.contains(&&symbol(&root.join("src/diamond-left.mts"), "diamondLeft")));
    assert!(direct_targets.contains(&&symbol(
        &root.join("src/diamond-right.mts"),
        "diamondRight"
    )));
    assert!(direct_targets.contains(&&symbol(&root.join("src/cycle-a.mts"), "cycleA")));

    assert!(
        graph
            .call_traces(&[entry_root.clone()], CallTraversal::Transitive, Some(0))
            .is_empty()
    );
    let depth_one = graph.call_traces(&[entry_root.clone()], CallTraversal::Transitive, Some(1));
    assert_eq!(depth_one, direct, "maxDepth=1 is the direct boundary");
    let depth_two = graph.call_traces(&[entry_root], CallTraversal::Transitive, Some(2));
    assert!(
        depth_two.iter().any(|trace| {
            trace.target == symbol(&root.join("src/diamond-shared.mts"), "shared")
        })
    );
    assert!(
        depth_two
            .iter()
            .any(|trace| { trace.target == symbol(&root.join("src/cycle-b.mts"), "cycleB") })
    );

    // A file traversal is deliberately a same-source-file layer, even when
    // the root function calls imported functions.
    assert!(
        graph
            .call_traces(
                &[symbol(&root.join("src/local.mts"), "first")],
                CallTraversal::File,
                None,
            )
            .iter()
            .all(|trace| trace.target.as_file() == Some(root.join("src/local.mts").as_path()))
    );
    assert!(
        graph
            .call_traces(&[symbol(&entry, "entry")], CallTraversal::File, None)
            .is_empty()
    );
}

#[test]
fn call_traversal_uses_deterministic_shortest_diamond_paths_and_terminates_cycles() {
    let (root, graph) = call_fixture_graph();
    let entry = root.join("src/entry.mts");
    let shared = symbol(&root.join("src/diamond-shared.mts"), "shared");
    let traces = graph.call_traces(&[symbol(&entry, "entry")], CallTraversal::Transitive, None);
    let shared_trace = traces
        .iter()
        .find(|trace| trace.target == shared)
        .expect("diamond target is reachable");
    assert_eq!(shared_trace.nodes.len(), 3);
    assert_eq!(
        shared_trace.nodes[1],
        symbol(&root.join("src/diamond-left.mts"), "diamondLeft")
    );
    assert_eq!(
        traces.iter().filter(|trace| trace.target == shared).count(),
        1,
        "diamond target has one shortest trace"
    );

    for name in ["cycleA", "cycleB", "cycleC"] {
        let path = root.join("src").join(format!(
            "cycle-{}.mts",
            name.trim_start_matches("cycle").to_ascii_lowercase()
        ));
        assert!(
            traces
                .iter()
                .any(|trace| trace.target == symbol(&path, name))
        );
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
    assert!(
        graph
            .expand_call_roots(&[CallRoot::File(global_file.clone())])
            .contains(&symbol(&global_file, "globalOnly"))
    );
    assert!(
        graph
            .expand_call_roots(&[CallRoot::Function {
                file: global_file.clone(),
                symbol: "notDefined".to_string(),
            }])
            .is_empty()
    );

    let unknown_file = root.join("src/unknown.mts");
    let unknown_root = symbol(&unknown_file, "unknown");
    assert!(
        graph
            .call_traces(&[unknown_root], CallTraversal::Transitive, None)
            .is_empty()
    );
    assert!(graph.resolved_call_sites().iter().any(|site| {
        site.file == unknown_file
            && site.source_callee == "globalThis.setTimeout"
            && matches!(site.target, ResolvedCallTarget::Unknown)
    }));
}

#[test]
fn call_resolution_follows_immutable_aliases_and_reexported_defaults_only() {
    let (root, graph) = call_fixture_graph();
    let aliases = root.join("src/aliases.mts");
    let target = symbol(&root.join("src/alias-target.mts"), "importedTarget");
    let default = symbol(&root.join("src/alias-target.mts"), "defaultTarget");
    let roots = graph.expand_call_roots(&[CallRoot::File(aliases.clone())]);
    let traces = graph.call_traces(&roots, CallTraversal::Direct, None);

    assert!(graph.call_traces(&[NodeId::file(&aliases)], CallTraversal::Direct, None).iter().any(
        |trace| matches!(&trace.target, NodeId::Symbol { symbol, .. } if symbol.starts_with("<anonymous:")),
    ));

    assert_eq!(
        traces.iter().filter(|trace| trace.target == target).count(),
        2,
        "the module and invoked anonymous callback each retain one canonical call edge"
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
            && site.source_callee == "second"
            && matches!(
                &site.target,
                ResolvedCallTarget::ModuleExport {
                    repository_target: Some((file, scope)),
                    ..
                } if file == &root.join("src/alias-target.mts") && scope == "importedTarget"
            )
    }));
    assert!(traces.iter().any(|trace| trace.target == default));
    assert!(
        !traces.iter().any(|trace| {
            trace.target == symbol(&root.join("src/mixed-star-callable.mts"), "collision")
        }),
        "a non-callable export-star collision must not select the callable branch"
    );
    assert!(
        !traces.iter().any(|trace| {
            trace.target
                == symbol(
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
        "cycle",
        "mutable",
        "mutableDeclaration",
        "cycleA",
        "collision",
        "externalCollision",
        "window.setTimeout",
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
