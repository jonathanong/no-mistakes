use super::tokens::{is_doc_marker, skip_to_newline};

#[derive(Default)]
pub(super) struct ScanState {
    pub(super) line_start: bool,
    pub(super) line_offset: usize,
    pub(super) entry_indent: usize,
    pub(super) after_indicator: bool,
    pub(super) candidate_key_col: Option<usize>,
    pub(super) pending_owner_indent: Option<usize>,
    pub(super) plain_indent: Option<usize>,
    pub(super) block_indent: Option<usize>,
    pub(super) flow_depth: usize,
    pub(super) at_token_start: bool,
    pub(super) after_json_like: bool,
    pub(super) after_tag: bool,
    pub(super) after_anchor: bool,
    pub(super) explicit_key_indent: Option<usize>,
}

impl ScanState {
    pub(super) fn new() -> Self {
        Self {
            line_start: true,
            at_token_start: true,
            ..Self::default()
        }
    }

    pub(super) fn clear_prefix(&mut self) {
        self.after_tag = false;
        self.after_anchor = false;
        self.explicit_key_indent = None;
    }

    pub(super) fn record_key_col(&mut self, col: usize) {
        if self.after_indicator && self.candidate_key_col.is_none() {
            self.candidate_key_col = Some(col);
        }
    }

    pub(super) fn expect_token(&mut self) {
        self.at_token_start = true;
        self.after_json_like = false;
        self.plain_indent = None;
        self.clear_prefix();
    }

    pub(super) fn mark_plain(&mut self, col: usize) {
        if self.at_token_start {
            if self.flow_depth == 0 {
                self.record_key_col(col);
                self.plain_indent = Some(self.entry_indent);
            }
            self.at_token_start = false;
        }
        self.pending_owner_indent = None;
        self.after_json_like = false;
        self.clear_prefix();
    }

    pub(super) fn finish_quoted(&mut self, next: Option<usize>, pos: usize) -> usize {
        self.record_key_col(pos - self.line_offset);
        self.at_token_start = false;
        self.pending_owner_indent = None;
        self.clear_prefix();
        if let Some(end) = next {
            self.after_json_like = true;
            self.plain_indent = None;
            end
        } else {
            self.after_json_like = false;
            pos + 1
        }
    }

    pub(super) fn advance_line_start(&mut self, bytes: &[u8], i: usize) -> usize {
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
            self.reset_block_line(spaces);
        }
        after
    }

    fn reset_block_line(&mut self, spaces: usize) {
        self.after_tag = false;
        if self.plain_indent.is_some_and(|owner| spaces > owner) {
            self.at_token_start = false;
            self.pending_owner_indent = None;
            self.clear_prefix();
        } else {
            self.plain_indent = None;
            self.at_token_start = true;
            self.candidate_key_col = None;
            if let Some(owner) = self.pending_owner_indent.take().filter(|&o| spaces > o) {
                self.entry_indent = owner;
                self.after_indicator = true;
            } else {
                self.entry_indent = spaces;
                self.after_indicator = false;
                self.after_anchor = false;
            }
            if self.explicit_key_indent.is_some_and(|ind| spaces < ind) {
                self.explicit_key_indent = None;
            }
        }
        self.after_json_like = false;
    }
}
