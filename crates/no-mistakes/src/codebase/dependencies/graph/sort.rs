fn add_via_kind(entry: &mut NodeEntry, kind: EdgeKind) {
    if !entry.via.contains(&kind) {
        entry.via.push(kind);
        entry.via.sort_by_key(|k| k.sort_key());
    }
}

#[cfg(test)]
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

fn cmp_node_sort_keys(left: &NodeId, right: &NodeId) -> std::cmp::Ordering {
    cmp_concatenated(&node_sort_parts(left), &node_sort_parts(right))
}

fn node_sort_parts(n: &NodeId) -> [std::borrow::Cow<'_, str>; 4] {
    match n {
        NodeId::File(path) => [path.to_string_lossy(), "".into(), "".into(), "".into()],
        NodeId::Symbol { file, symbol } => [
            file.to_string_lossy(),
            "#".into(),
            std::borrow::Cow::Borrowed(symbol),
            "".into(),
        ],
        NodeId::Module(specifier) => [
            "module:".into(),
            std::borrow::Cow::Borrowed(specifier),
            "".into(),
            "".into(),
        ],
        NodeId::QueueJob { queue_file, job } => [
            queue_file.to_string_lossy(),
            "#".into(),
            std::borrow::Cow::Borrowed(job),
            "".into(),
        ],
        NodeId::WorkflowJob { workflow_file, job } => [
            workflow_file.to_string_lossy(),
            "#job:".into(),
            std::borrow::Cow::Borrowed(job),
            "".into(),
        ],
        NodeId::WorkflowStep {
            workflow_file,
            job,
            step,
        } => [
            workflow_file.to_string_lossy(),
            "#job:".into(),
            std::borrow::Cow::Borrowed(job),
            format!("/step:{step}").into(),
        ],
    }
}

fn cmp_concatenated(
    left: &[std::borrow::Cow<'_, str>],
    right: &[std::borrow::Cow<'_, str>],
) -> std::cmp::Ordering {
    let left_bytes = left.iter().flat_map(|part| part.as_bytes());
    let right_bytes = right.iter().flat_map(|part| part.as_bytes());
    left_bytes.cmp(right_bytes)
}
