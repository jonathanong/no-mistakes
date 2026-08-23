use super::{into_adjacency_map, push_neighbor, push_ordinal, seed_known_targets, Adjacency};
use crate::fx::{fx_map, FxHashMap};

fn one_adj(key: &str, adj: Adjacency<String, u8>) -> FxHashMap<String, Adjacency<String, u8>> {
    let mut map = fx_map();
    map.insert(key.to_owned(), adj);
    map
}

#[test]
fn push_neighbor_hit_and_miss_keep_first_seen_ordinals() {
    for (existing, key, neighbor, ordinal, expected_neighbors, expected_ordinals) in [
        (None, "a", ("b", 1_u8), 0, vec![("b", 1)], vec![0]),
        (
            Some(("a", vec![("b", 1)], vec![0])),
            "a",
            ("c", 2),
            1,
            vec![("b", 1), ("c", 2)],
            vec![0, 1],
        ),
        (
            Some(("a", vec![("b", 1)], vec![0])),
            "z",
            ("c", 2),
            3,
            vec![("c", 2)],
            vec![3],
        ),
    ] {
        let mut map = match existing {
            Some((from, neighbors, ordinals)) => one_adj(
                from,
                Adjacency {
                    neighbors: neighbors
                        .into_iter()
                        .map(|(to, kind)| (to.to_owned(), kind))
                        .collect(),
                    ordinals,
                },
            ),
            None => fx_map(),
        };
        push_neighbor(
            &mut map,
            &key.to_owned(),
            (neighbor.0.to_owned(), neighbor.1),
            ordinal,
        );
        let adj = map.get(key).expect("key is present after push");
        assert_eq!(
            adj.neighbors,
            expected_neighbors
                .into_iter()
                .map(|(to, kind)| (to.to_owned(), kind))
                .collect::<Vec<_>>()
        );
        assert_eq!(adj.ordinals, expected_ordinals);
    }
}

#[test]
fn push_ordinal_hit_and_miss_do_not_clone_on_hit() {
    for (existing, key, ordinal, expected) in [
        (None, "a", 4, vec![4]),
        (Some(("a", vec![1])), "a", 2, vec![1, 2]),
        (Some(("a", vec![1])), "b", 7, vec![7]),
    ] {
        let mut map: FxHashMap<String, Adjacency<String, u8>> = match existing {
            Some((from, ordinals)) => one_adj(
                from,
                Adjacency {
                    neighbors: Vec::new(),
                    ordinals,
                },
            ),
            None => fx_map(),
        };
        push_ordinal(&mut map, &key.to_owned(), ordinal);
        assert_eq!(
            map.get(key).map(|adj| adj.ordinals.as_slice()),
            Some(expected.as_slice())
        );
    }
}

#[test]
fn seed_known_targets_covers_existing_and_missing_sources() {
    for (existing, expected_len) in [
        (None, 0),
        (
            Some(Adjacency {
                neighbors: vec![
                    ("b".to_owned(), 1_u8),
                    ("b".to_owned(), 2),
                    ("c".to_owned(), 1),
                ],
                ordinals: vec![0, 1, 2],
            }),
            2,
        ),
    ] {
        let known = seed_known_targets(existing.as_ref());
        assert_eq!(known.len(), expected_len);
        if expected_len == 2 {
            assert!(known
                .get("b")
                .is_some_and(|kinds| kinds.contains(&1) && kinds.contains(&2)));
            assert!(known.get("c").is_some_and(|kinds| kinds.contains(&1)));
        }
    }
}

#[test]
fn into_adjacency_map_preserves_keys_and_presizes_ordinals() {
    let mut input = fx_map();
    input.insert("a".to_owned(), vec![("b".to_owned(), 1_u8)]);
    input.insert("empty".to_owned(), Vec::new());
    let mapped = into_adjacency_map(input);
    assert_eq!(
        mapped.get("a").map(|adj| adj.neighbors.as_slice()),
        Some(&[("b".to_owned(), 1)][..])
    );
    assert_eq!(
        mapped.get("empty").map(|adj| adj.ordinals.capacity()),
        Some(0)
    );
    assert_eq!(mapped.get("a").map(|adj| adj.ordinals.capacity()), Some(1));
}

#[test]
fn adjacency_eq_slice_and_vec_compare_neighbors_only() {
    let adj = Adjacency {
        neighbors: vec![("b".to_owned(), 1_u8), ("c".to_owned(), 2)],
        ordinals: vec![9, 8],
    };
    let owned = vec![("b".to_owned(), 1_u8), ("c".to_owned(), 2)];
    let wrong = vec![("b".to_owned(), 1_u8)];

    assert!(adj.eq(owned.as_slice()));
    assert_eq!(adj, owned);
    assert!(!adj.eq(wrong.as_slice()));
    assert_ne!(adj, wrong);
}
