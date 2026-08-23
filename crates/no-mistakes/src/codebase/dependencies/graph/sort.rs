fn add_via_kind_to(via: &mut Vec<EdgeKind>, kind: EdgeKind) {
    if !via.contains(&kind) {
        via.push(kind);
        via.sort_by_key(|k| k.sort_key());
    }
}

fn add_via_kind(entry: &mut NodeEntry, kind: EdgeKind) {
    add_via_kind_to(&mut entry.via, kind);
}

/// Historical formatted sort key. Test oracle for `cached_node_sort_key`.
#[cfg_attr(not(test), allow(dead_code))]
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
        NodeId::TrpcProcedure {
            router_file,
            procedure,
        } => format!("{}#procedure:{procedure}", router_file.to_string_lossy()),
    }
}

include!("sort_key.rs");

/// File < Symbol < Module < QueueJob < … so display-key collisions keep
/// historical NodeId order without cloning the node into the cached key.
fn node_variant_rank(n: &NodeId) -> u8 {
    match n {
        NodeId::File(_) => 0,
        NodeId::Symbol { .. } => 1,
        NodeId::Module(_) => 2,
        NodeId::QueueJob { .. } => 3,
        NodeId::WorkflowJob { .. } => 4,
        NodeId::WorkflowStep { .. } => 5,
        NodeId::TrpcProcedure { .. } => 6,
    }
}

fn adjacency_sort_key(n: &NodeId, kind: EdgeKind) -> (NodeSortKey, u8, (u8, u8)) {
    (
        cached_node_sort_key(n),
        node_variant_rank(n),
        kind.sort_key(),
    )
}

fn cached_node_sort_key(n: &NodeId) -> NodeSortKey {
    match n {
        NodeId::File(path) => NodeSortKey::new(Some(path.clone_arc()), "", None, None),
        NodeId::Symbol { file, symbol } => {
            NodeSortKey::new(Some(file.clone_arc()), "#", Some(Arc::clone(symbol)), None)
        }
        NodeId::Module(specifier) => {
            NodeSortKey::new(None, "module:", Some(Arc::clone(specifier)), None)
        }
        NodeId::QueueJob { queue_file, job } => NodeSortKey::new(
            Some(queue_file.clone_arc()),
            "#",
            Some(Arc::clone(job)),
            None,
        ),
        NodeId::WorkflowJob { workflow_file, job } => NodeSortKey::new(
            Some(workflow_file.clone_arc()),
            "#job:",
            Some(Arc::clone(job)),
            None,
        ),
        NodeId::WorkflowStep {
            workflow_file,
            job,
            step,
        } => NodeSortKey::new(
            Some(workflow_file.clone_arc()),
            "#job:",
            Some(Arc::clone(job)),
            Some(*step),
        ),
        NodeId::TrpcProcedure {
            router_file,
            procedure,
        } => NodeSortKey::new(
            Some(router_file.clone_arc()),
            "#procedure:",
            Some(Arc::clone(procedure)),
            None,
        ),
    }
}

fn cmp_node_sort_keys(left: &NodeId, right: &NodeId) -> std::cmp::Ordering {
    let mut left_step = [0u8; 32];
    let mut right_step = [0u8; 32];
    cmp_concatenated(
        &node_sort_parts(left, &mut left_step),
        &node_sort_parts(right, &mut right_step),
    )
}

fn node_sort_parts<'a>(
    n: &'a NodeId,
    step_buf: &'a mut [u8; 32],
) -> [std::borrow::Cow<'a, str>; 4] {
    match n {
        NodeId::File(path) => [path.to_string_lossy(), "".into(), "".into(), "".into()],
        NodeId::Symbol { file, symbol } => [
            file.to_string_lossy(),
            "#".into(),
            std::borrow::Cow::Borrowed(symbol.as_ref()),
            "".into(),
        ],
        NodeId::Module(specifier) => [
            "module:".into(),
            std::borrow::Cow::Borrowed(specifier.as_ref()),
            "".into(),
            "".into(),
        ],
        NodeId::QueueJob { queue_file, job } => [
            queue_file.to_string_lossy(),
            "#".into(),
            std::borrow::Cow::Borrowed(job.as_ref()),
            "".into(),
        ],
        NodeId::WorkflowJob { workflow_file, job } => [
            workflow_file.to_string_lossy(),
            "#job:".into(),
            std::borrow::Cow::Borrowed(job.as_ref()),
            "".into(),
        ],
        NodeId::WorkflowStep {
            workflow_file,
            job,
            step,
        } => [
            workflow_file.to_string_lossy(),
            "#job:".into(),
            std::borrow::Cow::Borrowed(job.as_ref()),
            std::borrow::Cow::Borrowed(write_step_suffix(*step, step_buf)),
        ],
        NodeId::TrpcProcedure {
            router_file,
            procedure,
        } => [
            router_file.to_string_lossy(),
            "#procedure:".into(),
            std::borrow::Cow::Borrowed(procedure.as_ref()),
            "".into(),
        ],
    }
}

fn write_step_suffix(step: usize, buf: &mut [u8; 32]) -> &str {
    const PREFIX: &[u8] = b"/step:";
    buf[..PREFIX.len()].copy_from_slice(PREFIX);
    let digit_len = write_decimal(step, &mut buf[PREFIX.len()..]);
    std::str::from_utf8(&buf[..PREFIX.len() + digit_len]).expect("step suffix is ASCII")
}

fn write_decimal(mut value: usize, buf: &mut [u8]) -> usize {
    let mut digits = [0u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    let written = &digits[start..];
    buf[..written.len()].copy_from_slice(written);
    written.len()
}

fn cmp_concatenated(
    left: &[std::borrow::Cow<'_, str>],
    right: &[std::borrow::Cow<'_, str>],
) -> std::cmp::Ordering {
    left.iter()
        .flat_map(|part| part.as_bytes())
        .cmp(right.iter().flat_map(|part| part.as_bytes()))
}
