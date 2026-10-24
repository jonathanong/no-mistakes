use super::super::ancestry::{error_kinds, ErrorKind};
use super::*;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, GraphBuildPlan, NodeId};
use crate::codebase::ts_resolver::TsConfig;

fn graph() -> (PathBuf, DepGraph) {
    let root = fixture().parent().unwrap().join("uncertain/fixture");
    let config = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            extends: true,
            ..Default::default()
        },
    )
    .unwrap();
    (root, graph)
}

#[test]
fn scope_only_base_symbols_preserve_known_unknown_and_non_error_ancestry() {
    let (root, graph) = graph();
    let classes = graph.class_declarations();
    // Legacy symbol projections may omit parser IDs. Erase IDs on real fixture
    // edges rather than invent a TypeScript shape that the parser cannot emit.
    let forward = classes
        .iter()
        .map(|class| {
            let edges = graph
                .dependencies_of_node(&class.node())
                .into_iter()
                .flatten()
                .map(|(target, kind)| {
                    let target = match target {
                        NodeId::Symbol { file, symbol, .. } if *kind == EdgeKind::Extends => {
                            NodeId::symbol(file, symbol.as_ref())
                        }
                        _ => target.clone(),
                    };
                    (target, *kind)
                })
                .collect();
            (class.node(), edges)
        })
        .collect();
    let projected = crate::codebase::dependencies::graph::test_support::from_typed_maps(
        root,
        forward,
        Default::default(),
    );
    let original = error_kinds(&graph, classes);
    let projected = error_kinds(&projected, classes);
    assert!(
        projected == original,
        "scope-only projection changed ancestry"
    );
    for (name, expected) in [
        ("ChildError", ErrorKind::Error),
        ("ExternalChild", ErrorKind::Unknown),
        ("NotAnError", ErrorKind::Other),
    ] {
        let index = classes
            .iter()
            .position(|class| class.scope == name)
            .unwrap();
        assert!(projected[index] == expected, "{name}");
    }
}

#[test]
fn malformed_file_base_edges_leave_ancestry_unknown_without_crediting_a_class() {
    let (root, graph) = graph();
    let classes = graph.class_declarations();
    let index = classes
        .iter()
        .position(|class| class.scope == "ChildError")
        .unwrap();
    let child = &classes[index];
    // A mixed graph can contain a malformed typed edge. A File target is not
    // a class identity, so it must never establish error ancestry or crash.
    let forward = [(
        child.node(),
        vec![(NodeId::file(&child.file), EdgeKind::Extends)],
    )]
    .into_iter()
    .collect();
    let malformed = crate::codebase::dependencies::graph::test_support::from_typed_maps(
        root,
        forward,
        Default::default(),
    );
    assert!(error_kinds(&graph, classes)[index] == ErrorKind::Error);
    assert!(error_kinds(&malformed, classes)[index] == ErrorKind::Unknown);
}
