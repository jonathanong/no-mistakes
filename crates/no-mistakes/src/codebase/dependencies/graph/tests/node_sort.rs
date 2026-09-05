use super::super::*;

fn node_sort_table() -> [NodeId; 13] {
    [
        NodeId::file("/repo/a.ts"),
        NodeId::symbol("/repo/a.ts", "z"),
        NodeId::symbol("/repo/a.ts", "job"),
        NodeId::queue_job("/repo/a.ts", "job"),
        NodeId::queue_job("/repo/source", "job"),
        NodeId::file("/repo/source#job"),
        NodeId::workflow_job("/repo/a.ts", "build"),
        NodeId::workflow_step("/repo/a.ts", "build", 2),
        NodeId::workflow_step("/repo/a.ts", "build", 10),
        NodeId::trpc_procedure("/repo/a.ts", "user.get"),
        NodeId::Module("pkg".into()),
        NodeId::Module("other".into()),
        NodeId::file("module:pkg"),
    ]
}

#[test]
fn borrowed_node_sort_keys_match_formatted_string_order() {
    let nodes = node_sort_table();
    for left in &nodes {
        for right in &nodes {
            assert_eq!(
                cmp_node_sort_keys(left, right),
                node_sort_key(left).cmp(&node_sort_key(right)),
                "sort key mismatch for {left:?} vs {right:?}"
            );
        }
    }
}

#[test]
fn workflow_step_decimal_suffix_matches_formatted_string_order() {
    // Decimal digits must compare as the formatted "/step:{n}" string
    // (`/step:10` < `/step:2`), not as integers.
    let steps = [0, 1, 2, 9, 10, 11, 99, 100, 101, usize::MAX];
    let nodes: Vec<_> = steps
        .into_iter()
        .map(|step| NodeId::workflow_step("/repo/a.ts", "build", step))
        .collect();
    for left in &nodes {
        for right in &nodes {
            assert_eq!(
                cmp_node_sort_keys(left, right),
                node_sort_key(left).cmp(&node_sort_key(right)),
                "step suffix mismatch for {left:?} vs {right:?}"
            );
            assert_eq!(
                cached_node_sort_key(left).cmp(&cached_node_sort_key(right)),
                node_sort_key(left).cmp(&node_sort_key(right)),
                "cached step suffix mismatch for {left:?} vs {right:?}"
            );
        }
    }
}

#[test]
fn cached_node_sort_key_matches_formatted_string_order() {
    let nodes = node_sort_table();
    for left in &nodes {
        for right in &nodes {
            assert_eq!(
                cached_node_sort_key(left).cmp(&cached_node_sort_key(right)),
                node_sort_key(left).cmp(&node_sort_key(right)),
                "cached sort key mismatch for {left:?} vs {right:?}"
            );
            assert_eq!(
                cached_node_sort_key(left) == cached_node_sort_key(right),
                node_sort_key(left) == node_sort_key(right),
                "cached sort key eq mismatch for {left:?} vs {right:?}"
            );
            assert_eq!(
                cached_node_sort_key(left).partial_cmp(&cached_node_sort_key(right)),
                Some(node_sort_key(left).cmp(&node_sort_key(right))),
                "cached sort key partial_cmp mismatch for {left:?} vs {right:?}"
            );
        }
    }
}

fn assert_concatenated_chunks_match_flat_memcmp(left: &[&[u8]], right: &[&[u8]]) {
    let expected = left
        .iter()
        .copied()
        .flatten()
        .cmp(right.iter().copied().flatten());
    assert_eq!(
        cmp_concatenated_bytes(left, right),
        expected,
        "chunked memcmp mismatch for {left:?} vs {right:?}"
    );
    assert_eq!(
        cmp_concatenated_bytes(right, left),
        expected.reverse(),
        "chunked memcmp reverse mismatch for {right:?} vs {left:?}"
    );
}

#[test]
fn concatenated_byte_chunks_match_flat_memcmp() {
    assert_concatenated_chunks_match_flat_memcmp(&[b"ab", b"c"], &[b"a", b"bc"]);
    assert_concatenated_chunks_match_flat_memcmp(&[b"", b"abc"], &[b"abc", b""]);
    assert_concatenated_chunks_match_flat_memcmp(&[], &[b""]);
    assert_concatenated_chunks_match_flat_memcmp(&[b"a"], &[b"a", b"b"]);
    assert_concatenated_chunks_match_flat_memcmp(&[b"ab"], &[b"a"]);
    assert_concatenated_chunks_match_flat_memcmp(&[b"a", b"c"], &[b"ab"]);
    assert_concatenated_chunks_match_flat_memcmp(
        &[b"/repo/source", b"#", b"job"],
        &[b"/repo/source#job"],
    );
    assert_concatenated_chunks_match_flat_memcmp(&[b"/step:10"], &[b"/step:2"]);
    assert_concatenated_chunks_match_flat_memcmp(&[b"xx", b""], &[]);
    assert_concatenated_chunks_match_flat_memcmp(&[b"abc"], &[b"abd"]);
    assert_concatenated_chunks_match_flat_memcmp(&[b"ab", b"cd"], &[b"ab", b"ce"]);
    assert_concatenated_chunks_match_flat_memcmp(&[b"short", b"-tail"], &[b"sh", b"ort-tail"]);
    assert_concatenated_chunks_match_flat_memcmp(&[b"", b"", b"z"], &[b"z"]);
    assert_concatenated_chunks_match_flat_memcmp(&[b"prefix", b"rest"], &[b"pre", b"fixrest"]);
}

#[test]
fn flatten_source_cached_key_matches_formatted_order() {
    let nodes = node_sort_table();
    let mut cached = nodes.to_vec();
    cached.sort_by_cached_key(|node| (cached_node_sort_key(node), node.clone()));
    let mut formatted = nodes.to_vec();
    formatted.sort_by(|left, right| {
        node_sort_key(left)
            .cmp(&node_sort_key(right))
            .then_with(|| left.cmp(right))
    });
    assert_eq!(cached, formatted);
}

#[test]
fn cached_adjacency_sort_matches_formatted_node_id_key() {
    let kinds = [EdgeKind::Import, EdgeKind::Selector, EdgeKind::WorkflowStep];
    let mut pairs = Vec::new();
    for node in node_sort_table() {
        for kind in kinds {
            pairs.push((node.clone(), kind));
        }
    }
    let mut cached = pairs.clone();
    cached.sort_by_cached_key(|(n, k)| adjacency_sort_key(n, *k));
    let mut formatted = pairs;
    formatted.sort_by_cached_key(|(n, k)| (node_sort_key(n), n.clone(), k.sort_key()));
    assert_eq!(cached, formatted);
}
