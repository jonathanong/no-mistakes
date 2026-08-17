use super::*;
use std::cell::Cell;
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
        index.forward().get("a").map(|adj| adj.neighbors.as_slice()),
        Some(&[("b".to_owned(), 1), ("c".to_owned(), 2)][..])
    );
    assert_eq!(
        index.reverse().get("b").map(|adj| adj.neighbors.as_slice()),
        Some(&[("a".to_owned(), 1)][..])
    );
}

#[test]
fn unique_edge_constructor_retains_ordinals_and_sorts_adjacency() {
    let edges = vec![edge("a", "c", 2), edge("a", "b", 1), edge("b", "d", 3)];
    let index = EdgeIndex::from_unique_edges_in_order(edges.clone());

    assert_eq!(index.edges(), edges);
    assert_eq!(
        index.forward().get("a").map(|adj| adj.neighbors.as_slice()),
        Some(&[("b".to_owned(), 1), ("c".to_owned(), 2)][..]),
        "ordinal order must be independent from adjacency ordering",
    );
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "already-unique edge input must not contain duplicates")]
fn unique_edge_constructor_rejects_duplicate_edges_in_debug_builds() {
    EdgeIndex::from_unique_edges_in_order(vec![edge("a", "b", 1), edge("a", "b", 1)]);
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
fn prepared_projection_preserves_aliases_orientation_empty_roots_and_first_seen_order() {
    fn public_node(node: &str) -> String {
        match node {
            "job-a" | "job-b" => "job".to_owned(),
            "worker-a" | "worker-b" => "worker".to_owned(),
            node => node.to_owned(),
        }
    }

    let prepared = PreparedRelationshipIndex::from_edges(
        [
            edge("producer", "job-a", 1),
            edge("producer", "job-b", 1),
            edge("job-a", "worker-a", 2),
            edge("job-b", "worker-b", 2),
        ],
        |node: &String| public_node(node),
    );
    let project = |edge: &CanonicalEdge<String, u8>, from: &str, to: &str| {
        (from.to_owned(), to.to_owned(), edge.kind)
    };

    assert_eq!(
        prepared.edge_view(&[], None, project),
        vec![
            ("job".to_owned(), "worker".to_owned(), 2),
            ("producer".to_owned(), "job".to_owned(), 1),
        ],
        "empty edge roots retain the full public edge view order",
    );
    assert_eq!(
        prepared.edge_view(&["producer".to_owned()], None, project),
        vec![
            ("producer".to_owned(), "job".to_owned(), 1),
            ("job".to_owned(), "worker".to_owned(), 2),
        ],
        "aliases reached after the first hop must project only their first public edge",
    );
    assert_eq!(
        prepared.related(&["worker".to_owned()], EdgeDirection::Dependents, project,),
        vec![
            ("worker".to_owned(), "job".to_owned(), 2),
            ("job".to_owned(), "producer".to_owned(), 1),
        ],
        "reverse views retain traversal orientation before callers sort them",
    );
    assert!(
        prepared
            .related(&[], EdgeDirection::Both, project)
            .is_empty(),
        "related queries retain their historical empty-root result",
    );
}

#[test]
fn prepared_index_renders_each_typed_node_once_before_preserving_public_order() {
    let calls = Cell::new(0);
    let prepared = PreparedRelationshipIndex::from_edges(
        [
            edge("shared", "z", 1),
            edge("shared", "a", 1),
            edge("other", "shared", 2),
            edge("shared", "z", 1),
        ],
        |node: &String| {
            calls.set(calls.get() + 1);
            match node.as_str() {
                "shared" => "job".to_owned(),
                "a" | "z" => "worker".to_owned(),
                "other" => "producer".to_owned(),
                _ => unreachable!("test nodes are exhaustive"),
            }
        },
    );

    assert_eq!(calls.get(), 4, "one public name per distinct typed node");
    assert_eq!(
        prepared.edges(),
        &[
            edge("shared", "a", 1),
            edge("shared", "z", 1),
            edge("other", "shared", 2)
        ],
        "public from/to/kind order must retain typed-edge ordering as its final tie breaker",
    );
    assert_eq!(
        prepared.edge_view(&[], None, |edge, from, to| {
            (from.to_owned(), to.to_owned(), edge.kind)
        }),
        vec![
            ("job".to_owned(), "worker".to_owned(), 1),
            ("producer".to_owned(), "job".to_owned(), 2),
        ],
        "projection must reuse cached public names and retain first-seen public edges",
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
fn direct_adjacency_constructor_and_invariant_cover_both_map_states() {
    fn compare_edges(
        left: &CanonicalEdge<String, u8>,
        right: &CanonicalEdge<String, u8>,
    ) -> std::cmp::Ordering {
        left.cmp(right)
    }

    let mut forward = HashMap::new();
    forward.insert("a".to_owned(), vec![("b".to_owned(), 1_u8)]);
    let mut reverse = HashMap::new();
    reverse.insert("b".to_owned(), vec![("a".to_owned(), 1_u8)]);

    let index = EdgeIndex::from_adjacency_maps_by(forward.clone(), reverse.clone(), compare_edges);

    assert_eq!(index.edges(), &[edge("a", "b", 1)]);

    reverse.get_mut("b").unwrap().push(("c".to_owned(), 2_u8));
    let panic = std::panic::catch_unwind(|| {
        #[cfg(coverage)]
        test_support::assert_adjacency_maps_are_consistent(&forward, &reverse);
        // A panicking generic constructor call makes LLVM subtract downstream
        // LCOV counters to zero, so coverage builds exercise the same invariant
        // directly while ordinary tests retain the constructor contract.
        #[cfg(not(coverage))]
        EdgeIndex::from_adjacency_maps_by(forward, reverse, compare_edges);
    })
    .expect_err("reverse-only edge must be rejected");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or_default();
    assert!(message.contains("forward and reverse adjacency maps must describe identical edges"));
}
