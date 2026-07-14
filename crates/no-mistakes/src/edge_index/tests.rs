use super::*;
use std::collections::HashMap;

fn edge(from: &str, to: &str, kind: u8) -> CanonicalEdge<String, u8> {
    CanonicalEdge::new(from.to_owned(), to.to_owned(), kind)
}

fn index(edges: &[(&str, &str, u8)]) -> EdgeIndex<String, u8> {
    EdgeIndex::from_edges(edges.iter().map(|(from, to, kind)| edge(from, to, *kind)))
}

#[test]
fn canonicalizes_edges_and_sorts_adjacency() {
    let index = index(&[("a", "c", 2), ("a", "b", 1), ("a", "b", 1)]);
    assert_eq!(index.edges(), &[edge("a", "c", 2), edge("a", "b", 1)]);
    assert_eq!(
        index.forward().get("a").unwrap(),
        &[("b".to_owned(), 1), ("c".to_owned(), 2)]
    );
    assert_eq!(index.reverse().get("b").unwrap(), &[("a".to_owned(), 1)]);
}

#[test]
fn dependencies_are_level_ordered_by_original_ordinal() {
    let index = index(&[("b", "d", 1), ("a", "c", 1), ("a", "b", 1), ("c", "e", 1)]);
    assert_eq!(
        index.traverse(&["a".into()], EdgeDirection::Dependencies, None),
        vec![
            edge("a", "c", 1),
            edge("a", "b", 1),
            edge("b", "d", 1),
            edge("c", "e", 1),
        ]
    );
}

#[test]
fn depth_zero_one_and_unlimited_are_distinct() {
    let index = index(&[("a", "b", 1), ("b", "c", 1)]);
    assert!(index
        .traverse(&["a".into()], EdgeDirection::Dependencies, Some(0))
        .is_empty());
    assert_eq!(
        index.traverse(&["a".into()], EdgeDirection::Dependencies, Some(1)),
        vec![edge("a", "b", 1)]
    );
    assert_eq!(
        index.traverse(&["a".into()], EdgeDirection::Dependencies, None),
        vec![edge("a", "b", 1), edge("b", "c", 1)]
    );
}

#[test]
fn cycles_and_back_edges_are_retained_once() {
    let index = index(&[("a", "b", 1), ("b", "a", 2), ("b", "c", 3)]);
    assert_eq!(
        index.traverse(&["a".into()], EdgeDirection::Dependencies, None),
        vec![edge("a", "b", 1), edge("b", "a", 2), edge("b", "c", 3),]
    );
}

#[test]
fn overlapping_multi_roots_do_not_duplicate_edges() {
    let index = index(&[("a", "c", 1), ("b", "c", 2), ("c", "d", 3)]);
    assert_eq!(
        index.traverse(&["a".into(), "b".into()], EdgeDirection::Dependencies, None,),
        vec![edge("a", "c", 1), edge("b", "c", 2), edge("c", "d", 3),]
    );
}

#[test]
fn reverse_and_both_return_traversal_orientations() {
    let index = index(&[("a", "b", 1), ("c", "b", 2)]);
    assert_eq!(
        index.traverse(&["b".into()], EdgeDirection::Dependents, None),
        vec![edge("b", "a", 1), edge("b", "c", 2)]
    );
    assert_eq!(
        index.traverse(&["a".into()], EdgeDirection::Both, None),
        vec![
            edge("a", "b", 1),
            edge("b", "a", 1),
            edge("b", "c", 2),
            edge("c", "b", 2),
        ]
    );
}

#[test]
fn unknown_roots_have_no_edges() {
    let index = index(&[("a", "b", 1)]);
    assert!(index
        .traverse(&["missing".into()], EdgeDirection::Both, None)
        .is_empty());
}

#[test]
fn aliases_expand_every_reached_frontier_without_synthetic_edges() {
    let index = index(&[
        ("p1", "j1", 1),
        ("j1", "w1", 2),
        ("p2", "j2", 1),
        ("j2", "w2", 2),
    ]);
    let aliases = NodeAliases::from_groups([vec!["j1".to_owned(), "j2".to_owned()]]);

    assert_eq!(
        index.traverse_with_aliases(&["p1".into()], EdgeDirection::Dependencies, None, &aliases,),
        vec![
            edge("p1", "j1", 1),
            edge("j1", "w1", 2),
            edge("j2", "w2", 2)
        ]
    );
    assert_eq!(
        index.traverse_with_aliases(&["w1".into()], EdgeDirection::Dependents, None, &aliases),
        vec![
            edge("w1", "j1", 2),
            edge("j1", "p1", 1),
            edge("j2", "p2", 1)
        ]
    );
}

#[test]
fn both_deduplicates_self_loops_and_reciprocal_projections() {
    let index = index(&[("a", "a", 1), ("a", "b", 2), ("b", "a", 2)]);
    assert_eq!(
        index.traverse(&["a".into()], EdgeDirection::Both, None),
        vec![edge("a", "a", 1), edge("a", "b", 2), edge("b", "a", 2),]
    );
}

#[test]
#[should_panic(expected = "forward and reverse adjacency maps must describe identical edges")]
fn direct_adjacency_constructor_rejects_reverse_only_edges() {
    let mut forward = HashMap::new();
    forward.insert("a".to_owned(), vec![("b".to_owned(), 1_u8)]);
    let mut reverse = HashMap::new();
    reverse.insert(
        "b".to_owned(),
        vec![("a".to_owned(), 1_u8), ("c".to_owned(), 2_u8)],
    );

    let _ = EdgeIndex::from_adjacency_maps_by(forward, reverse, |left, right| left.cmp(right));
}
