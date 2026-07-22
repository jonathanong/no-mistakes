fn add_via_kind(entry: &mut NodeEntry, kind: EdgeKind) {
    if !entry.via.contains(&kind) {
        entry.via.push(kind);
        entry.via.sort_by_key(|k| k.sort_key());
    }
}

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
