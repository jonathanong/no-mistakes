#[test]
fn borrowed_node_sort_keys_match_formatted_string_order() {
    let nodes = [
        NodeId::file("/repo/a.ts"),
        NodeId::symbol("/repo/a.ts", "z"),
        NodeId::symbol("/repo/a.ts", "job"),
        NodeId::queue_job("/repo/a.ts", "job"),
        NodeId::queue_job("/repo/source", "job"),
        NodeId::file("/repo/source#job"),
        NodeId::workflow_job("/repo/a.ts", "build"),
        NodeId::workflow_step("/repo/a.ts", "build", 2),
        NodeId::workflow_step("/repo/a.ts", "build", 10),
        NodeId::Module("pkg".into()),
        NodeId::Module("other".into()),
        NodeId::file("module:pkg"),
    ];
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
        }
    }
}
