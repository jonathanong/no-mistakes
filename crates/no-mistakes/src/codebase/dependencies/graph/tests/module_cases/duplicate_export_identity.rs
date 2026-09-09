use super::*;
use crate::codebase::dependencies::extract::CallableId;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn call_graph() -> (PathBuf, DepGraph) {
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
            imports: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    (root, graph)
}

fn target_ids(graph: &DepGraph, file: &Path, symbol: &str) -> HashSet<CallableId> {
    graph
        .expand_call_roots(&[CallRoot::Function {
            file: file.to_path_buf(),
            symbol: symbol.to_string(),
        }])
        .into_iter()
        .filter_map(|node| match node {
            NodeId::Symbol {
                callable_id: Some(id),
                ..
            } => Some(id),
            _ => None,
        })
        .collect()
}

fn has_file_symbol(node: &NodeId, file: &Path, symbol: &str) -> bool {
    matches!(
        node,
        NodeId::Symbol {
            file: target_file,
            symbol: target_symbol,
            ..
        } if target_file.as_ref() == file && target_symbol.as_ref() == symbol
    )
}

#[test]
fn duplicate_display_exports_select_the_module_binding_identity() {
    let (root, graph) = call_graph();
    let file = root.join("src/duplicate-exported-callable.mts");
    let facts = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            function_calls: true,
            imports: true,
            ..TsFactPlan::default()
        },
    );
    let file_facts = facts.get(&file).expect("duplicate export fixture");
    let target_ids_in_file = file_facts
        .callable_scope_ids
        .iter()
        .filter(|(_, scope)| *scope == "target")
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();
    assert_eq!(
        target_ids_in_file.len(),
        2,
        "the fixture must keep two same-spelled target callables: {file_facts:#?}"
    );

    let exported = target_ids(&graph, &file, "target");
    let aliased = target_ids(&graph, &file, "run");
    assert_eq!(exported.len(), 1);
    assert_eq!(exported, aliased);

    let traces = graph.call_traces(
        &graph.expand_call_roots(&[CallRoot::Function {
            file: file.clone(),
            symbol: "run".to_string(),
        }]),
        CallTraversal::Transitive,
        None,
    );
    assert!(traces
        .iter()
        .any(|trace| has_file_symbol(&trace.target, &file, "actualLeaf")));
    assert!(!traces
        .iter()
        .any(|trace| has_file_symbol(&trace.target, &file, "decoyLeaf")));
}

#[test]
fn named_reexport_and_star_keep_the_same_exported_identity() {
    let (root, graph) = call_graph();
    let source = root.join("src/duplicate-exported-callable.mts");
    let expected = target_ids(&graph, &source, "run");
    assert_eq!(expected.len(), 1);

    let reexport = root.join("src/duplicate-exported-callable-reexport.mts");
    let star = root.join("src/duplicate-exported-callable-star.mts");
    assert_eq!(target_ids(&graph, &reexport, "run"), expected);
    assert_eq!(target_ids(&graph, &star, "run"), expected);
    assert_eq!(target_ids(&graph, &star, "target"), expected);
}

#[test]
fn imported_duplicate_export_calls_follow_the_canonical_identity() {
    let (root, graph) = call_graph();
    let consumer = root.join("src/duplicate-exported-callable-consumer.mts");
    let source = root.join("src/duplicate-exported-callable.mts");
    let expected = target_ids(&graph, &source, "target");
    let expected_id = expected.iter().copied().next().expect("exported identity");

    for symbol in ["callRun", "callTarget"] {
        let traces = graph.call_traces(
            &graph.expand_call_roots(&[CallRoot::Function {
                file: consumer.clone(),
                symbol: symbol.to_string(),
            }]),
            CallTraversal::Direct,
            None,
        );
        assert_eq!(traces.len(), 1, "{symbol}: {traces:#?}");
        assert!(matches!(
            &traces[0].target,
            NodeId::Symbol {
                file,
                symbol,
                callable_id: Some(id),
            } if file.as_ref() == source.as_path() && symbol.as_ref() == "target" && *id == expected_id
        ));
        assert!(
            !traces
                .iter()
                .any(|trace| has_file_symbol(&trace.target, &source, "decoyLeaf"))
        );
    }

    let transitive = graph.call_traces(
        &graph.expand_call_roots(&[CallRoot::Function {
            file: consumer,
            symbol: "callRun".to_string(),
        }]),
        CallTraversal::Transitive,
        None,
    );
    assert!(transitive
        .iter()
        .any(|trace| has_file_symbol(&trace.target, &source, "actualLeaf")));
}

#[test]
fn imported_duplicate_export_reachability_follows_the_canonical_identity() {
    let (root, graph) = call_graph();
    let source = root.join("src/duplicate-exported-callable-imports.mts");
    let deps = graph.deps_of(
        &[NodeId::file(source)],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    assert!(deps
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/called.mts").as_path())));
    assert!(!deps
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/uncalled.mts").as_path())));
}
