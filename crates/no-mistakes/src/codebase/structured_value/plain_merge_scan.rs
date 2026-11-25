mod state;
mod tokens;

use state::ScanState;
use tokens::{
    is_comment_start, is_mapping_colon, is_plain_merge_candidate, is_space_or_end,
    skip_double_quoted, skip_node_property, skip_single_quoted, skip_to_newline,
    starts_block_scalar,
};

/// Returns `true` when `source` contains any unquoted, uncommented plain `<<`
/// token in a position that could act as a YAML 1.1 merge key (direct mapping
/// key, explicit `? <<` key, or anchored `&name <<` scalar).
pub(super) fn has_plain_merge_key(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut i = usize::from(bytes.starts_with(b"\xef\xbb\xbf")) * 3;
    let mut st = ScanState::new();

    while i < bytes.len() {
        if st.line_start {
            i = st.advance_line_start(bytes, i);
            continue;
        }
        if bytes[i] == b'<' && is_merge_at(&st, bytes, i) {
            return true;
        }
        i = step_byte(&mut st, bytes, i);
    }
    false
}

fn step_byte(st: &mut ScanState, bytes: &[u8], i: usize) -> usize {
    match bytes[i] {
        b'\n' => {
            st.line_start = true;
            i + 1
        }
        b' ' | b'\t' | b'\r' => i + 1,
        b'#' if is_comment_start(bytes, i) => {
            st.plain_indent = None;
            st.line_start = true;
            skip_to_newline(bytes, i)
        }
        b'\'' | b'"' if st.at_token_start => step_quote(st, bytes, i),
        b'|' | b'>' => step_block_scalar(st, bytes, i).unwrap_or_else(|| {
            st.mark_plain(i - st.line_offset);
            i + 1
        }),
        b'{' | b'[' | b'}' | b']' | b',' if step_flow(st, bytes[i]) => i + 1,
        b':' if is_mapping_colon(bytes, i, st.flow_depth, st.after_json_like) => {
            step_colon(st);
            i + 1
        }
        b'-' | b'?' if step_indicator(st, bytes, i) => i + 1,
        b'&' | b'!' if st.at_token_start => step_property(st, bytes, i),
        b'<' if i + 1 < bytes.len() && bytes[i + 1] == b'<' => {
            st.mark_plain(i - st.line_offset);
            i + 2
        }
        _ => {
            st.mark_plain(i - st.line_offset);
            i + 1
        }
    }
}

fn step_quote(st: &mut ScanState, bytes: &[u8], i: usize) -> usize {
    let next = if bytes[i] == b'\'' {
        skip_single_quoted(bytes, i)
    } else {
        skip_double_quoted(bytes, i)
    };
    st.finish_quoted(next, i)
}

fn step_block_scalar(st: &mut ScanState, bytes: &[u8], i: usize) -> Option<usize> {
    if st.flow_depth == 0 && st.at_token_start && starts_block_scalar(bytes, i) {
        st.block_indent = Some(st.entry_indent);
        st.plain_indent = None;
        st.pending_owner_indent = None;
        st.clear_prefix();
        st.line_start = true;
        return Some(skip_to_newline(bytes, i));
    }
    None
}

fn step_flow(st: &mut ScanState, byte: u8) -> bool {
    match byte {
        b'{' | b'[' if st.at_token_start || st.flow_depth > 0 => {
            st.flow_depth += 1;
            st.pending_owner_indent = None;
            st.expect_token();
            true
        }
        b'}' | b']' if st.flow_depth > 0 => {
            st.flow_depth -= 1;
            st.at_token_start = false;
            st.after_json_like = true;
            st.clear_prefix();
            true
        }
        b',' if st.flow_depth > 0 => {
            st.expect_token();
            true
        }
        _ => false,
    }
}

fn step_colon(st: &mut ScanState) {
    if let Some(col) = st.candidate_key_col.take() {
        st.entry_indent = col;
    }
    st.after_indicator = false;
    st.pending_owner_indent = (st.flow_depth == 0).then_some(st.entry_indent);
    st.expect_token();
}

fn step_indicator(st: &mut ScanState, bytes: &[u8], i: usize) -> bool {
    let explicit = bytes[i] == b'?';
    if !st.at_token_start || (st.flow_depth > 0 && !explicit) || !is_space_or_end(bytes, i + 1) {
        return false;
    }
    let col = i - st.line_offset;
    if st.flow_depth == 0 {
        st.entry_indent = col;
        st.after_indicator = true;
        st.candidate_key_col = None;
        st.pending_owner_indent = Some(col);
    }
    st.expect_token();
    if explicit {
        st.explicit_key_indent = Some(col);
    }
    true
}

fn step_property(st: &mut ScanState, bytes: &[u8], i: usize) -> usize {
    st.record_key_col(i - st.line_offset);
    let next = skip_node_property(bytes, i, st.flow_depth > 0);
    if bytes[i] == b'!' {
        st.after_tag = true;
    } else if next > i + 1 {
        st.after_anchor = true;
    }
    st.after_json_like = false;
    next
}

fn is_merge_at(st: &ScanState, bytes: &[u8], i: usize) -> bool {
    let explicit_or_anchored = st.after_anchor || st.explicit_key_indent.is_some();
    i + 1 < bytes.len()
        && bytes[i + 1] == b'<'
        && st.at_token_start
        && !st.after_tag
        && is_plain_merge_candidate(bytes, i, st.flow_depth, explicit_or_anchored)
}
