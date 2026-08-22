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
