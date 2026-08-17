use super::*;

fn edge(from: &str, to: &str, kind: u8) -> CanonicalEdge<String, u8> {
    CanonicalEdge::new(from.to_owned(), to.to_owned(), kind)
}

#[test]
fn extend_edges_preserving_ordinals_matches_rebuild_cases() {
    for (label, base, incoming, expected_edges, expected_a_ordinals) in [
        (
            "empty incoming leaves base ordinals",
            vec![edge("a", "b", 1)],
            vec![],
            vec![edge("a", "b", 1)],
            Some(&[0][..]),
        ),
        (
            "new source appends after base",
            vec![edge("a", "b", 1)],
            vec![edge("c", "d", 2)],
            vec![edge("a", "b", 1), edge("c", "d", 2)],
            Some(&[0][..]),
        ),
        (
            "existing source new target keeps first-seen",
            vec![edge("a", "b", 1)],
            vec![edge("a", "c", 2)],
            vec![edge("a", "b", 1), edge("a", "c", 2)],
            Some(&[0, 1][..]),
        ),
        (
            "duplicate of existing is skipped",
            vec![edge("a", "b", 1)],
            vec![edge("a", "b", 1), edge("a", "c", 2)],
            vec![edge("a", "b", 1), edge("a", "c", 2)],
            Some(&[0, 1][..]),
        ),
        (
            "duplicate within incoming batch is skipped",
            vec![edge("a", "b", 1)],
            vec![edge("c", "d", 2), edge("c", "d", 2)],
            vec![edge("a", "b", 1), edge("c", "d", 2)],
            Some(&[0][..]),
        ),
        (
            "same endpoints different kind is accepted",
            vec![edge("a", "b", 1)],
            vec![edge("a", "b", 2)],
            vec![edge("a", "b", 1), edge("a", "b", 2)],
            Some(&[0, 1][..]),
        ),
        (
            "new source after isolated node",
            vec![],
            vec![edge("a", "b", 1)],
            vec![edge("a", "b", 1)],
            Some(&[0][..]),
        ),
    ] {
        let rebuilt = EdgeIndex::from_edges_and_nodes(
            base.iter().cloned().chain(incoming.iter().cloned()),
            std::iter::empty(),
        );
        let mut appended = EdgeIndex::from_edges_and_nodes(base, std::iter::empty());
        appended.extend_edges_preserving_ordinals(incoming);
        assert_eq!(appended.edges(), expected_edges, "{label}");
        assert_eq!(appended.edges(), rebuilt.edges(), "{label}");
        assert_eq!(
            appended
                .forward()
                .get("a")
                .map(|adj| adj.ordinals.as_slice()),
            expected_a_ordinals,
            "{label}",
        );
        assert_eq!(
            appended.traverse(&["a".into()], EdgeDirection::Dependencies, Some(1)),
            rebuilt.traverse(&["a".into()], EdgeDirection::Dependencies, Some(1)),
            "{label}",
        );
    }
}

#[test]
fn extend_edges_from_seeded_empty_source_hits_existing_adjacency() {
    let mut index =
        EdgeIndex::from_edges_and_nodes(std::iter::empty(), ["a".to_owned(), "orphan".to_owned()]);
    index.extend_edges_preserving_ordinals([edge("a", "b", 1), edge("a", "b", 1)]);
    assert_eq!(index.edges(), &[edge("a", "b", 1)]);
    assert_eq!(
        index.forward().get("a").map(|adj| adj.ordinals.as_slice()),
        Some(&[0][..])
    );
    assert!(index
        .forward()
        .get("orphan")
        .is_some_and(|adj| adj.neighbors.is_empty()));
}

#[test]
fn extend_edges_records_reverse_ordinals_for_new_targets() {
    let mut index = EdgeIndex::from_edges([edge("a", "b", 1)]);
    index.extend_edges_preserving_ordinals([edge("c", "b", 2), edge("c", "d", 3)]);
    assert_eq!(
        index.reverse().get("b").map(|adj| adj.ordinals.as_slice()),
        Some(&[0, 1][..])
    );
    assert_eq!(
        index.reverse().get("d").map(|adj| adj.ordinals.as_slice()),
        Some(&[2][..])
    );
}
