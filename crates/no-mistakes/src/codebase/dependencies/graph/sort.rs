fn add_via_kind(entry: &mut NodeEntry, kind: EdgeKind) {
    if !entry.via.contains(&kind) {
        entry.via.push(kind);
        entry.via.sort_by_key(|k| k.sort_key());
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
    let left_bytes = left.iter().flat_map(|part| part.as_bytes());
    let right_bytes = right.iter().flat_map(|part| part.as_bytes());
    left_bytes.cmp(right_bytes)
}
