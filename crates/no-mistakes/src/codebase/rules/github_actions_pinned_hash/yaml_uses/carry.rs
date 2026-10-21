/// Flow nesting carried across lines. A `with` or `env` mapping that breaks
/// after `{` still hides the `uses` keys inside it.
#[derive(Default)]
pub(super) struct Carry {
    quote: Option<char>,
    token: String,
    scalar: String,
    after_colon: bool,
    key_was_uses: bool,
    flow_depth: i32,
    flow_input: Vec<bool>,
    value_key_is_input: bool,
    skip_anchor: bool,
    alias: bool,
    block_header: bool,
}

pub(super) struct LineMark {
    pub starts_in_flow_input: bool,
    pub present_count: usize,
    pub awaits: bool,
    pub opens_block: bool,
    pub block_uses: bool,
}

impl Carry {
    pub(super) fn scan_line(&mut self, line: &str) -> LineMark {
        let starts_in_flow_input = self.in_flow_input();
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
        if self.quote.is_none() {
            self.reset_token();
        }
        LineMark {
            starts_in_flow_input,
            present_count: hits,
            awaits,
            opens_block: header,
            block_uses,
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
    }
}

fn finish_scalar(scalar: &str, key_was_uses: bool) -> bool {
    key_was_uses && !scalar.is_empty()
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
