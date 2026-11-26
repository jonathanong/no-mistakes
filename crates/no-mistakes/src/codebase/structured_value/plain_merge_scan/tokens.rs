pub(super) fn skip_to_newline(bytes: &[u8], i: usize) -> usize {
    bytes[i..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(bytes.len(), |n| i + n + 1)
}

pub(super) fn is_space_or_end(bytes: &[u8], pos: usize) -> bool {
    pos >= bytes.len() || matches!(bytes[pos], b' ' | b'\t' | b'\n' | b'\r')
}

pub(super) fn is_comment_start(bytes: &[u8], pos: usize) -> bool {
    pos == 0 || matches!(bytes[pos - 1], b' ' | b'\t' | b'\n' | b'\r')
}

pub(super) fn is_doc_marker(bytes: &[u8], pos: usize) -> bool {
    pos + 3 <= bytes.len()
        && matches!(&bytes[pos..pos + 3], b"---" | b"...")
        && is_space_or_end(bytes, pos + 3)
}

pub(super) fn is_mapping_colon(
    bytes: &[u8],
    i: usize,
    flow_depth: usize,
    after_json: bool,
) -> bool {
    after_json
        || is_space_or_end(bytes, i + 1)
        || (flow_depth > 0 && matches!(bytes[i + 1], b',' | b']' | b'}' | b'[' | b'{'))
}

pub(super) fn skip_node_property(bytes: &[u8], mut i: usize, in_flow: bool) -> usize {
    if bytes[i] == b'!' && bytes.get(i + 1) == Some(&b'<') {
        i += 2;
        while i < bytes.len() && !matches!(bytes[i], b'>' | b'\n' | b'\r') {
            i += 1;
        }
        return if bytes.get(i) == Some(&b'>') {
            i + 1
        } else {
            i
        };
    }
    while i < bytes.len()
        && !matches!(bytes[i], b' ' | b'\t' | b'\n' | b'\r')
        && !(in_flow && matches!(bytes[i], b',' | b'[' | b']' | b'{' | b'}'))
    {
        i += 1;
    }
    i
}

pub(super) fn skip_single_quoted(bytes: &[u8], mut i: usize) -> Option<usize> {
    i += 1;
    while i < bytes.len() {
        if bytes[i] == b'\'' {
            i += 1;
            if bytes.get(i) == Some(&b'\'') {
                i += 1;
                continue;
            }
            return Some(i);
        }
        i += 1;
    }
    None
}

pub(super) fn skip_double_quoted(bytes: &[u8], mut i: usize) -> Option<usize> {
    i += 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
            continue;
        }
        if bytes[i] == b'"' {
            return Some(i + 1);
        }
        i += 1;
    }
    None
}

pub(super) fn starts_block_scalar(bytes: &[u8], pos: usize) -> bool {
    let mut i = pos + 1;
    while i < bytes.len() && matches!(bytes[i], b'-' | b'+' | b'1'..=b'9') {
        i += 1;
    }
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t') {
        i += 1;
    }
    i >= bytes.len() || matches!(bytes[i], b'\r' | b'\n' | b'#')
}

pub(super) fn is_plain_merge_candidate(
    bytes: &[u8],
    pos: usize,
    flow_depth: usize,
    explicit_or_anchored: bool,
) -> bool {
    let after = pos + 2;
    let right_ok =
        is_space_or_end(bytes, after) || matches!(bytes[after], b':' | b',' | b'}' | b']');
    if !right_ok {
        return false;
    }
    explicit_or_anchored || followed_by_mapping_colon(bytes, after, flow_depth)
}

fn followed_by_mapping_colon(bytes: &[u8], mut i: usize, flow_depth: usize) -> bool {
    loop {
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t') {
            i += 1;
        }
        if flow_depth > 0 && i < bytes.len() {
            if matches!(bytes[i], b'\r' | b'\n') {
                i += 1;
                continue;
            }
            if bytes[i] == b'#' && i > 0 && matches!(bytes[i - 1], b' ' | b'\t' | b'\n' | b'\r') {
                i = skip_to_newline(bytes, i);
                continue;
            }
        }
        break;
    }
    i < bytes.len() && bytes[i] == b':' && is_mapping_colon(bytes, i, flow_depth, false)
}
