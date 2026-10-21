use super::{finish_scalar, Carry};

impl Carry {
    /// Returns true when the rest of this line cannot change the flow stack.
    pub(super) fn consume(&mut self, ch: char, hits: &mut usize) -> bool {
        if let Some(active) = self.quote {
            return self.consume_quoted(ch, active, hits);
        }
        if self.anchor_pending && !ch.is_whitespace() && ch != '{' {
            self.clear_anchor_watch();
        }
        if matches!(ch, '"' | '\'') {
            self.comment_break = false;
            self.quote = Some(ch);
            if !self.after_colon {
                self.token.clear();
            }
            return false;
        }
        // `foo#bar` keeps the hash. A comment needs whitespace before `#`.
        if ch == '#' && self.comment_break {
            return true;
        }
        if self.consume_anchor(ch, hits) {
            self.comment_break = ch.is_whitespace();
            return false;
        }
        if matches!(ch, '{' | '[') {
            self.open_flow(ch == '[');
            return false;
        }
        if matches!(ch, '}' | ']' | ',') {
            self.close_flow(ch, hits);
            return false;
        }
        if let Some(stop) = self.consume_space(ch, hits) {
            self.comment_break = true;
            return stop;
        }
        self.comment_break = false;
        if ch == ':' && !self.after_colon {
            self.finish_key();
            return false;
        }
        if self.after_colon
            && self.scalar.is_empty()
            && self.flow_depth == 0
            && matches!(ch, '|' | '>')
        {
            self.block_header = true;
            self.place.clear_pending();
            self.scalar.push(ch);
            return false;
        }
        if self.after_colon {
            self.value_key_is_input = false;
            self.place.clear_pending();
            self.scalar.push(ch);
        } else {
            self.value_key_is_input = false;
            // `foo uses:` — a new word replaces the key. `uses :` keeps it.
            if self.key_gap {
                self.token.clear();
                self.key_gap = false;
            }
            self.token.push(ch);
        }
        false
    }

    fn consume_quoted(&mut self, ch: char, active: char, hits: &mut usize) -> bool {
        if self.escape_hex > 0 {
            self.push_hex_digit(ch);
            return false;
        }
        if self.escape_quote {
            self.escape_quote = false;
            self.begin_escape(ch);
            return false;
        }
        // `\"` stays inside the scalar, so a later `uses` is still a key.
        if active == '"' && ch == '\\' {
            self.escape_quote = true;
            return false;
        }
        if ch != active {
            self.push_quoted(ch);
            return false;
        }
        self.quote = None;
        if self.after_colon && finish_scalar(&self.scalar, self.key_was_uses) {
            *hits += 1;
            // Clear before stopping so the end of the line does not count again.
            self.scalar.clear();
            self.key_was_uses = false;
            self.after_colon = false;
            return self.flow_depth == 0;
        }
        if self.after_colon {
            self.scalar.clear();
            self.key_was_uses = false;
            self.after_colon = false;
        }
        false
    }

    pub(super) fn push_quoted(&mut self, ch: char) {
        if self.after_colon {
            self.scalar.push(ch);
        } else {
            self.token.push(ch);
        }
    }

    fn open_flow(&mut self, sequence: bool) {
        let entering = self.in_flow_input() || self.value_key_is_input;
        self.flow_input.push(entering);
        self.value_key_is_input = false;
        self.flow_depth += 1;
        self.place.open_flow(sequence);
        self.token.clear();
        self.scalar.clear();
        self.after_colon = false;
        self.key_was_uses = false;
        self.key_gap = false;
        self.arm_anchor_mapping(sequence);
    }

    fn close_flow(&mut self, ch: char, hits: &mut usize) {
        if (self.alias && self.key_was_uses) || finish_scalar(&self.scalar, self.key_was_uses) {
            *hits += 1;
        }
        if ch != ',' {
            self.flow_depth -= 1;
            self.flow_input.pop();
            self.place.close_flow();
            if self.anchor_depth > self.flow_depth {
                self.anchor_depth = 0;
                self.anchor_name.clear();
            }
        }
        self.value_key_is_input = false;
        self.token.clear();
        self.scalar.clear();
        self.after_colon = false;
        self.key_was_uses = false;
        self.alias = false;
        self.skip_anchor = false;
    }

    /// `Some` when this character is a word break. The bool stops the line.
    fn consume_space(&mut self, ch: char, hits: &mut usize) -> Option<bool> {
        let word_break = ch.is_whitespace()
            || (ch == '-' && !self.after_colon && self.flow_depth == 0 && self.token.is_empty());
        if !word_break {
            return None;
        }
        // `run: | # explain` — the space before the comment is not a new scalar.
        if self.block_header && ch.is_whitespace() {
            return Some(false);
        }
        if self.after_colon && !self.scalar.is_empty() {
            if finish_scalar(&self.scalar, self.key_was_uses) {
                *hits += 1;
            }
            // Clear before stopping so the end of the line does not count again.
            let stop = self.flow_depth == 0;
            self.scalar.clear();
            self.key_was_uses = false;
            self.after_colon = false;
            return Some(stop);
        } else if !self.after_colon && !self.token.is_empty() {
            self.key_gap = true;
        }
        Some(false)
    }

    fn finish_key(&mut self) {
        let in_input = self.in_flow_input();
        self.value_key_is_input = !in_input && (self.token == "with" || self.token == "env");
        let action = self.place.note_key(&self.token);
        if self.token == "uses" && !action && !in_input && self.place.nested() {
            self.rejected_uses = true;
        }
        self.note_anchor_uses();
        self.key_was_uses = action && !in_input;
        self.key_gap = false;
        self.token.clear();
        self.scalar.clear();
        self.after_colon = true;
    }
}
