use super::{hops, reach_path};
use crate::codebase::dependencies::graph::test_support::from_typed_maps;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};
use crate::fx::FxHashMap;
use std::path::PathBuf;

#[test]
fn duplicate_gate_keeps_the_shortest_load_path() {
    let root = PathBuf::from("/repo");
    let gate = root.join("gate.mts");
    let left = root.join("a.mts");
    let right = root.join("b.mts");
    let dest = root.join("dest.mts");
    let forward = FxHashMap::from_iter([
        (
            NodeId::file(&gate),
            vec![
                (NodeId::module("pkg"), EdgeKind::Import),
                (NodeId::file(&left), EdgeKind::Import),
                (NodeId::file(&right), EdgeKind::Import),
            ],
        ),
        (
            NodeId::file(&left),
            vec![(NodeId::file(&dest), EdgeKind::Import)],
        ),
        (
            NodeId::file(&right),
            vec![(NodeId::file(&dest), EdgeKind::Import)],
        ),
    ]);
    let graph = from_typed_maps(root.clone(), forward, FxHashMap::default());
    let path = reach_path(&graph, &[gate.clone(), gate], &dest, &root);
    assert_eq!(path, "gate.mts → a.mts → dest.mts");
}

#[test]
fn a_gate_without_edges_names_no_path() {
    let root = PathBuf::from("/repo");
    let graph = from_typed_maps(root.clone(), FxHashMap::default(), FxHashMap::default());
    let path = reach_path(
        &graph,
        &[root.join("missing.mts")],
        &root.join("other.mts"),
        &root,
    );
    assert!(path.is_empty());
}

#[test]
fn hops_counts_load_edges_and_is_zero_when_unreached() {
    let root = PathBuf::from("/repo");
    let gate = root.join("gate.mts");
    let mid = root.join("mid.mts");
    let again = root.join("again.mts");
    let dest = root.join("dest.mts");
    let leaf = root.join("leaf.mts");
    let forward = FxHashMap::from_iter([
        (
            NodeId::file(&gate),
            vec![
                (NodeId::file(&mid), EdgeKind::Import),
                (NodeId::file(&again), EdgeKind::Import),
                (NodeId::file(&leaf), EdgeKind::Import),
                (NodeId::module("pkg"), EdgeKind::Import),
                (NodeId::file(&dest), EdgeKind::Call),
            ],
        ),
        (
            NodeId::file(&mid),
            vec![(NodeId::file(&dest), EdgeKind::Import)],
        ),
        (
            NodeId::file(&again),
            vec![(NodeId::file(&mid), EdgeKind::Import)],
        ),
    ]);
    let graph = from_typed_maps(root.clone(), forward, FxHashMap::default());
    assert_eq!(hops(&graph, &[gate.clone(), gate.clone()], &dest), 2);
    assert_eq!(hops(&graph, &[gate], &root.join("other.mts")), 0);
}
