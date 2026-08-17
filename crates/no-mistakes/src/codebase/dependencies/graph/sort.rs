fn add_via_kind(entry: &mut NodeEntry, kind: EdgeKind) {
    if !entry.via.contains(&kind) {
        entry.via.push(kind);
        entry.via.sort_by_key(|k| k.sort_key());
    }
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
    }
}

fn adjacency_sort_key(n: &NodeId, kind: EdgeKind) -> (NodeSortKey, u8, (u8, u8)) {
    (cached_node_sort_key(n), node_variant_rank(n), kind.sort_key())
}

fn cached_node_sort_key(n: &NodeId) -> NodeSortKey {
    match n {
        NodeId::File(path) => NodeSortKey::new(Some(Arc::clone(path)), "", None, None),
        NodeId::Symbol { file, symbol } => {
            NodeSortKey::new(Some(Arc::clone(file)), "#", Some(Arc::clone(symbol)), None)
        }
        NodeId::Module(specifier) => {
            NodeSortKey::new(None, "module:", Some(Arc::from(specifier.as_str())), None)
        }
        NodeId::QueueJob { queue_file, job } => NodeSortKey::new(
            Some(Arc::clone(queue_file)),
            "#",
            Some(Arc::clone(job)),
            None,
        ),
        NodeId::WorkflowJob { workflow_file, job } => NodeSortKey::new(
            Some(Arc::clone(workflow_file)),
            "#job:",
            Some(Arc::clone(job)),
            None,
        ),
        NodeId::WorkflowStep {
            workflow_file,
            job,
            step,
        } => NodeSortKey::new(
            Some(Arc::clone(workflow_file)),
            "#job:",
            Some(Arc::clone(job)),
            Some(*step),
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
            std::borrow::Cow::Borrowed(specifier),
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
