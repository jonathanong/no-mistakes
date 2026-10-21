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
}

pub(super) struct LineMark {
    pub starts_in_flow_input: bool,
    pub kind: LineKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum LineKind {
    None,
    Present,
    Awaits,
}

impl Carry {
    pub(super) fn scan_line(&mut self, line: &str) -> LineMark {
        let starts_in_flow_input = self.in_flow_input();
        let mut present = false;
        for ch in line.chars() {
            if self.consume(ch, &mut present) {
                break;
            }
        }
        if (self.alias && self.key_was_uses) || finish_scalar(&self.scalar, self.key_was_uses) {
            present = true;
        }
        // A quoted scalar that already has text stays on this line.
        if present && self.quote.is_some() {
            self.key_was_uses = false;
        }
        let awaits = !present
            && self.key_was_uses
            && self.scalar.is_empty()
            && self.flow_depth > 0
            && !self.in_flow_input();
        if self.quote.is_none() {
            self.reset_token();
        }
        let kind = if present {
            LineKind::Present
        } else if awaits {
            LineKind::Awaits
        } else {
            LineKind::None
        };
        LineMark {
            starts_in_flow_input,
            kind,
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
    }
}

fn finish_scalar(scalar: &str, key_was_uses: bool) -> bool {
    key_was_uses && !scalar.is_empty()
}

#[path = "carry_chars.rs"]
mod chars;
