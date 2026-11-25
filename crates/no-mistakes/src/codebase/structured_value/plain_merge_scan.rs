mod tokens;

use tokens::{
    is_doc_marker, is_mapping_colon, is_plain_merge_candidate, is_space_or_end, skip_double_quoted,
    skip_node_property, skip_single_quoted, skip_to_newline, starts_block_scalar,
};

#[derive(Default)]
struct ScanState {
    line_start: bool,
    line_offset: usize,
    entry_indent: usize,
    after_indicator: bool,
    candidate_key_col: Option<usize>,
    plain_indent: Option<usize>,
    block_indent: Option<usize>,
    flow_depth: usize,
    at_token_start: bool,
    after_json_like: bool,
}

impl ScanState {
    fn record_key_col(&mut self, col: usize) {
        if self.after_indicator && self.candidate_key_col.is_none() {
            self.candidate_key_col = Some(col);
        }
    }

    fn expect_token(&mut self) {
        self.at_token_start = true;
        self.after_json_like = false;
        self.plain_indent = None;
    }

    fn mark_plain(&mut self, col: usize) {
        if self.at_token_start {
            if self.flow_depth == 0 {
                self.record_key_col(col);
                self.plain_indent = Some(self.entry_indent);
            }
            self.at_token_start = false;
        }
        self.after_json_like = false;
    }

    fn finish_quoted(&mut self, next: Option<usize>, pos: usize) -> usize {
        self.record_key_col(pos - self.line_offset);
        self.at_token_start = false;
        if let Some(end) = next {
            self.after_json_like = true;
            self.plain_indent = None;
            end
        } else {
            self.after_json_like = false;
            pos + 1
        }
    }

    fn advance_line_start(&mut self, bytes: &[u8], i: usize) -> usize {
        if self.block_indent.is_none() && self.flow_depth == 0 && bytes[i] == b'%' {
            return skip_to_newline(bytes, i);
        }
        let spaces = bytes[i..].iter().take_while(|&&b| b == b' ').count();
        let after = i + spaces;
        if let Some(hdr) = self.block_indent {
            if after >= bytes.len() || matches!(bytes[after], b'\r' | b'\n') || spaces > hdr {
                return skip_to_newline(bytes, after);
            }
            self.block_indent = None;
        }
        if after >= bytes.len() || matches!(bytes[after], b'\r' | b'\n') {
            return skip_to_newline(bytes, after);
        }
        if bytes[after] == b'#' {
            self.plain_indent = None;
            return skip_to_newline(bytes, after);
        }
        self.line_offset = i;
        self.line_start = false;
        if spaces == 0 && is_doc_marker(bytes, after) {
            *self = Self {
                line_offset: i,
                at_token_start: true,
                ..Self::default()
            };
            return after + 3;
        }
        if self.flow_depth == 0 {
            if self.plain_indent.is_some_and(|owner| spaces > owner) {
                self.at_token_start = false;
            } else {
                self.expect_token();
                self.entry_indent = spaces;
                self.candidate_key_col = None;
                self.after_indicator = false;
            }
            self.after_json_like = false;
        }
        after
    }
}

/// Returns `true` when `source` contains any unquoted, uncommented plain `<<`
/// token in a position that could act as a YAML 1.1 merge key (direct mapping
/// key, explicit `? <<` key, or anchored `&name <<` scalar).
pub(super) fn has_plain_merge_key(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut i = usize::from(bytes.starts_with(b"\xef\xbb\xbf")) * 3;
    let mut st = ScanState {
        line_start: true,
        at_token_start: true,
        ..ScanState::default()
    };

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
                i = skip_to_newline(bytes, i);
                st.line_start = true;
            }
            b'{' | b'[' if st.at_token_start || st.flow_depth > 0 => {
                st.flow_depth += 1;
                st.expect_token();
                i += 1;
            }
            b'}' | b']' if st.flow_depth > 0 => {
                st.flow_depth -= 1;
                st.at_token_start = false;
                st.after_json_like = true;
                i += 1;
            }
            b',' if st.flow_depth > 0 => {
                st.at_token_start = true;
                st.after_json_like = false;
                i += 1;
            }
            b':' if is_mapping_colon(bytes, i, st.flow_depth, st.after_json_like) => {
                if let Some(col) = st.candidate_key_col.take() {
                    st.entry_indent = col;
                }
                st.after_indicator = false;
                st.expect_token();
                i += 1;
            }
            b'-' | b'?'
                if st.at_token_start
                    && (st.flow_depth == 0 || bytes[i] == b'?')
                    && is_space_or_end(bytes, i + 1) =>
            {
                if st.flow_depth == 0 {
                    st.entry_indent = i - st.line_offset;
                    st.after_indicator = true;
                    st.candidate_key_col = None;
                }
                st.expect_token();
                i += 1;
            }
            b'&' | b'!' if st.at_token_start => {
                st.record_key_col(i - st.line_offset);
                i = skip_node_property(bytes, i, st.flow_depth > 0);
                st.after_json_like = false;
            }
            b'<' if i + 1 < bytes.len() && bytes[i + 1] == b'<' => {
                if is_plain_merge_candidate(bytes, i) {
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
