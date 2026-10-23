use super::*;

fn fixture_graph() -> (PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-source-nodes"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            imports: true,
            ..Default::default()
        },
    )
    .unwrap();
    let imported = graph.deps_of(
        &[NodeId::file(root.join("calls.ts"))],
        Some(1),
        Some(&[EdgeKind::Import].into()),
    );
    assert!(
        imported
            .iter()
            .any(|entry| entry.node == NodeId::file(root.join("unreachable.ts"))),
        "fixture must contain non-call edges to test traversal filtering"
    );
    (root, graph)
}

fn function_root(graph: &DepGraph, root: &Path, name: &str) -> Vec<NodeId> {
    graph.expand_call_roots(&[CallRoot::Function {
        file: root.join("calls.ts"),
        symbol: name.to_string(),
    }])
}

#[test]
fn source_union_matches_materialized_traces_at_every_depth_boundary() {
    let (root, graph) = fixture_graph();
    let mut overlapping = function_root(&graph, &root, "alpha");
    overlapping.extend(function_root(&graph, &root, "beta"));
    let mut nested = function_root(&graph, &root, "alpha");
    nested.extend(function_root(&graph, &root, "tail"));
    assert_eq!(
        function_root(&graph, &root, "aliasAlpha"),
        function_root(&graph, &root, "alpha")
    );
    let file_roots = graph.expand_call_roots(&[CallRoot::File(root.join("calls.ts"))]);
    let module_roots = graph.expand_call_roots(&[CallRoot::Module(root.join("calls.ts"))]);
    for roots in [
        Vec::new(),
        overlapping.clone(),
        nested,
        file_roots,
        module_roots,
    ] {
        for traversal in [
            CallTraversal::Direct,
            CallTraversal::File,
            CallTraversal::Transitive,
        ] {
            for depth in [Some(0), Some(1), Some(2), Some(3), Some(4), None] {
                let mut expected: FxHashSet<NodeId> = roots.iter().cloned().collect();
                if traversal != CallTraversal::File {
                    let cap = if traversal == CallTraversal::Direct {
                        1
                    } else {
                        depth.unwrap_or(usize::MAX)
                    };
                    // The old trace projection is an independent compatibility oracle.
                    for trace in graph.call_traces(&roots, traversal, depth) {
                        if trace.nodes.len().saturating_sub(1) < cap {
                            expected.insert(trace.target);
                        }
                    }
                }
                assert_eq!(
                    graph.call_source_nodes(&roots, traversal, depth),
                    expected,
                    "traversal={traversal:?}, depth={depth:?}, roots={roots:?}"
                );
            }
        }
    }
    let selected = graph.call_source_nodes(&overlapping, CallTraversal::Transitive, Some(2));
    assert!(selected.contains(&function_root(&graph, &root, "shared")[0]));
    assert!(!selected.contains(&function_root(&graph, &root, "tail")[0]));
    assert!(!selected.contains(&function_root(&graph, &root, "isolated")[0]));
}

#[test]
fn overlapping_roots_expand_a_shared_cyclic_tail_once() {
    let (root, graph) = fixture_graph();
    let mut roots = function_root(&graph, &root, "alpha");
    roots.extend(function_root(&graph, &root, "beta"));
    // Duplicate selectors must not add a second queue entry either.
    roots.extend(function_root(&graph, &root, "alpha"));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let selected = crate::diagnostics::with_observer(Some(observer.clone()), || {
        graph.call_source_nodes(&roots, CallTraversal::Transitive, None)
    });
    for name in ["alpha", "beta", "shared", "tail", "leaf"] {
        assert!(
            selected.contains(&function_root(&graph, &root, name)[0]),
            "{name}"
        );
    }
    assert!(!selected.contains(&function_root(&graph, &root, "isolated")[0]));
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("graph.call_source_nodes.expanded"),
        Some(&(selected.len() as u64)),
        "each reachable source must be expanded once across all roots, not once per trace"
    );
}
