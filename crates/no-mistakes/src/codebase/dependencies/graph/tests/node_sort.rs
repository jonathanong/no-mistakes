fn node_sort_key(n: &NodeId) -> String {
    match n {
        NodeId::File(p) => p.to_string_lossy().into_owned(),
        NodeId::Symbol { file, symbol } => format!("{}#{symbol}", file.to_string_lossy()),
        NodeId::Module(specifier) => format!("module:{specifier}"),
        NodeId::QueueJob { queue_file, job } => {
            format!("{}#{}", queue_file.to_string_lossy(), job)
        }
        NodeId::WorkflowJob { workflow_file, job } => {
            format!("{}#job:{job}", workflow_file.to_string_lossy())
        }
        NodeId::WorkflowStep {
            workflow_file,
            job,
            step,
        } => format!("{}#job:{job}/step:{step}", workflow_file.to_string_lossy()),
    }
}

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
