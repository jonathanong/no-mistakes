use super::place::PlaceStack;

/// Flow nesting carried across lines. A `with` or `env` mapping that breaks
/// after `{` still hides the `uses` keys inside it. `uses` is an action only
/// on a job or a step.
#[derive(Default)]
pub(super) struct Carry {
    quote: Option<char>,
    /// The previous character in a double-quoted scalar was `\`.
    escape_quote: bool,
    /// Remaining hex digits of a `\x`, `\u`, or `\U` escape.
    escape_hex: u8,
    escape_value: u32,
    token: String,
    scalar: String,
    after_colon: bool,
    key_was_uses: bool,
    flow_depth: i32,
    flow_input: Vec<bool>,
    value_key_is_input: bool,
    key_gap: bool,
    skip_anchor: bool,
    alias: bool,
    /// `&name` whose mapping is being read so a later step alias can be judged.
    reading_anchor: bool,
    anchor_pending: bool,
    block_anchor: bool,
    step_alias: bool,
    anchor_depth: i32,
    anchor_indent: usize,
    anchor_name: String,
    alias_name: String,
    anchors_with_uses: Vec<String>,
    block_header: bool,
    place: PlaceStack,
    rejected_uses: bool,
}

pub(super) struct LineMark {
    pub starts_in_flow_input: bool,
    pub present_count: usize,
    pub awaits: bool,
    pub opens_block: bool,
    pub block_uses: bool,
    pub rejects_uses: bool,
}

impl Carry {
    pub(super) fn scan_line(&mut self, line: &str) -> LineMark {
        self.rejected_uses = false;
        if self.quote.is_none() && is_doc_start(line) {
            self.place.reset();
            self.flow_depth = 0;
            self.flow_input.clear();
            self.value_key_is_input = false;
            self.reset_anchor_state();
            self.reset_token();
            return empty_mark();
        }
        if self.quote.is_none() {
            self.end_block_anchor(line);
        }
        let starts_in_flow_input = self.in_flow_input();
        if self.quote.is_none() && self.flow_depth == 0 {
            self.place.begin_line(line);
        }
        let mut hits = 0usize;
        for ch in line.chars() {
            if self.consume(ch, &mut hits) {
                break;
            }
        }
        let header = self.block_header && is_block_header(&self.scalar);
        let block_uses = header && self.key_was_uses && !self.in_flow_input();
        let finished =
            (self.alias && self.key_was_uses) || finish_scalar(&self.scalar, self.key_was_uses);
        if !header && finished {
            hits += 1;
            // A quoted scalar that already has text stays on this line.
            if self.quote.is_some() {
                self.key_was_uses = false;
            }
        }
        let awaits = !header
            && self.key_was_uses
            && self.scalar.is_empty()
            && self.flow_depth > 0
            && !self.in_flow_input();
        if self.step_alias && self.anchor_has_uses() {
            hits += 1;
        }
        if self.quote.is_none() {
            // `&name` at the end of the line anchors the following block mapping.
            if self.reading_anchor && !self.anchor_name.is_empty() && self.anchor_depth == 0 {
                self.block_anchor = true;
            }
            self.reset_token();
        } else {
            // A trailing `\` escapes the line break, not the next character.
            self.escape_quote = false;
        }
        LineMark {
            starts_in_flow_input,
            present_count: hits,
            awaits,
            opens_block: header,
            block_uses,
            rejects_uses: self.rejected_uses,
        }
    }

    fn in_flow_input(&self) -> bool {
        self.flow_input.last().copied().unwrap_or(false)
    }

    fn reset_token(&mut self) {
        self.token.clear();
        self.scalar.clear();
        self.after_colon = false;
        self.key_was_uses = false;
        self.skip_anchor = false;
        self.alias = false;
        self.block_header = false;
        self.escape_quote = false;
        self.escape_hex = 0;
        self.escape_value = 0;
        self.key_gap = false;
        self.reading_anchor = false;
        self.step_alias = false;
        self.alias_name.clear();
    }
}

#[path = "carry_anchor.rs"]
mod anchor;

fn finish_scalar(scalar: &str, key_was_uses: bool) -> bool {
    key_was_uses && !scalar.is_empty()
}

fn is_doc_start(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("---") else {
        return false;
    };
    rest.trim().is_empty()
}

fn empty_mark() -> LineMark {
    LineMark {
        starts_in_flow_input: false,
        present_count: 0,
        awaits: false,
        opens_block: false,
        block_uses: false,
        rejects_uses: false,
    }
}

/// `|`, `>`, and the one optional chomp or indent digit YAML allows after them.
pub(super) fn is_block_header(scalar: &str) -> bool {
    let bytes = scalar.as_bytes();
    if bytes.first().is_none_or(|ch| *ch != b'|' && *ch != b'>') {
        return false;
    }
    let mut digit = false;
    let mut chomp = false;
    for ch in bytes.iter().skip(1) {
        match ch {
            b'+' | b'-' if !chomp => chomp = true,
            b'1'..=b'9' if !digit => digit = true,
            _ => return false,
        }
    }
    true
}

#[path = "carry_chars.rs"]
mod chars;
