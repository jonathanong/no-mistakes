mod state;
mod tokens;

use state::ScanState;
use tokens::{
    is_mapping_colon, is_plain_merge_candidate, is_space_or_end, skip_double_quoted,
    skip_node_property, skip_single_quoted, skip_to_newline, starts_block_scalar,
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
        match bytes[i] {
            b'\n' => {
                i += 1;
                st.line_start = true;
            }
            b' ' | b'\t' | b'\r' => i += 1,
            b'#' if i == 0 || matches!(bytes[i - 1], b' ' | b'\t' | b'\n' | b'\r') => {
                st.plain_indent = None;
                i = skip_to_newline(bytes, i);
                st.line_start = true;
            }
            b'\'' if st.at_token_start => i = st.finish_quoted(skip_single_quoted(bytes, i), i),
            b'"' if st.at_token_start => i = st.finish_quoted(skip_double_quoted(bytes, i), i),
            b'|' | b'>'
                if st.flow_depth == 0 && st.at_token_start && starts_block_scalar(bytes, i) =>
            {
                st.block_indent = Some(st.entry_indent);
                st.plain_indent = None;
                st.pending_owner_indent = None;
                st.clear_prefix();
                i = skip_to_newline(bytes, i);
                st.line_start = true;
            }
            b'{' | b'[' if st.at_token_start || st.flow_depth > 0 => {
                st.flow_depth += 1;
                st.pending_owner_indent = None;
                st.expect_token();
                i += 1;
            }
            b'}' | b']' if st.flow_depth > 0 => {
                st.flow_depth -= 1;
                st.at_token_start = false;
                st.after_json_like = true;
                st.clear_prefix();
                i += 1;
            }
            b',' if st.flow_depth > 0 => {
                st.expect_token();
                i += 1;
            }
            b':' if is_mapping_colon(bytes, i, st.flow_depth, st.after_json_like) => {
                if let Some(col) = st.candidate_key_col.take() {
                    st.entry_indent = col;
                }
                st.after_indicator = false;
                st.pending_owner_indent = (st.flow_depth == 0).then_some(st.entry_indent);
                st.expect_token();
                i += 1;
            }
            b'-' | b'?'
                if st.at_token_start
                    && (st.flow_depth == 0 || bytes[i] == b'?')
                    && is_space_or_end(bytes, i + 1) =>
            {
                let explicit = bytes[i] == b'?';
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
                i += 1;
            }
            b'&' | b'!' if st.at_token_start => {
                st.record_key_col(i - st.line_offset);
                let next = skip_node_property(bytes, i, st.flow_depth > 0);
                if bytes[i] == b'!' {
                    st.after_tag = true;
                } else if next > i + 1 {
                    st.after_anchor = true;
                }
                st.after_json_like = false;
                i = next;
            }
            b'<' if i + 1 < bytes.len() && bytes[i + 1] == b'<' => {
                let explicit_or_anchored = st.after_anchor || st.explicit_key_indent.is_some();
                if st.at_token_start
                    && !st.after_tag
                    && is_plain_merge_candidate(bytes, i, st.flow_depth, explicit_or_anchored)
                {
                    return true;
                }
                st.mark_plain(i - st.line_offset);
                i += 2;
            }
            _ => {
                st.mark_plain(i - st.line_offset);
                i += 1;
            }
        }
    }
    false
}
