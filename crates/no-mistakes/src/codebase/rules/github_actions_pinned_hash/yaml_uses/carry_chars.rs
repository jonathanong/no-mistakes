use super::{finish_scalar, Carry};

impl Carry {
    /// Returns true when the rest of this line cannot change the flow stack.
    pub(super) fn consume(&mut self, ch: char, present: &mut bool) -> bool {
        if let Some(active) = self.quote {
            return self.consume_quoted(ch, active, present);
        }
        if matches!(ch, '"' | '\'') {
            self.quote = Some(ch);
            if !self.after_colon {
                self.token.clear();
            }
            return false;
        }
        if ch == '#' {
            return true;
        }
        if self.consume_anchor(ch, present) {
            return false;
        }
        if matches!(ch, '{' | '[') {
            self.open_flow();
            return false;
        }
        if matches!(ch, '}' | ']' | ',') {
            self.close_flow(ch, present);
            return false;
        }
        if let Some(stop) = self.consume_space(ch, present) {
            return stop;
        }
        if ch == ':' && !self.after_colon {
            self.finish_key();
            return false;
        }
        if self.after_colon {
            self.value_key_is_input = false;
            self.scalar.push(ch);
        } else {
            self.value_key_is_input = false;
            self.token.push(ch);
        }
        false
    }

    fn consume_quoted(&mut self, ch: char, active: char, present: &mut bool) -> bool {
        if ch != active {
            if self.after_colon {
                self.scalar.push(ch);
            } else {
                self.token.push(ch);
            }
            return false;
        }
        self.quote = None;
        if self.after_colon && finish_scalar(&self.scalar, self.key_was_uses) {
            *present = true;
            if self.flow_depth == 0 {
                return true;
            }
        }
        if self.after_colon {
            self.scalar.clear();
            self.key_was_uses = false;
            self.after_colon = false;
        }
        false
    }

    /// An anchor or alias sits between `uses:` and the scalar. Returns true
    /// when this character was consumed as part of that name.
    fn consume_anchor(&mut self, ch: char, present: &mut bool) -> bool {
        if self.skip_anchor {
            let boundary = ch.is_whitespace() || matches!(ch, '}' | ']' | ',' | ':');
            if !boundary {
                return true;
            }
            let was_alias = self.alias && self.key_was_uses;
            self.skip_anchor = false;
            self.alias = false;
            if was_alias {
                *present = true;
                self.key_was_uses = false;
            }
            return ch.is_whitespace();
        }
        if self.after_colon && self.scalar.is_empty() && matches!(ch, '&' | '*') {
            self.alias = ch == '*';
            self.skip_anchor = true;
            return true;
        }
        false
    }

    fn open_flow(&mut self) {
        let entering = self.in_flow_input() || self.value_key_is_input;
        self.flow_input.push(entering);
        self.value_key_is_input = false;
        self.flow_depth += 1;
        self.token.clear();
        self.scalar.clear();
        self.after_colon = false;
        self.key_was_uses = false;
    }

    fn close_flow(&mut self, ch: char, present: &mut bool) {
        if (self.alias && self.key_was_uses) || finish_scalar(&self.scalar, self.key_was_uses) {
            *present = true;
        }
        if ch != ',' {
            self.flow_depth -= 1;
            self.flow_input.pop();
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
    fn consume_space(&mut self, ch: char, present: &mut bool) -> Option<bool> {
        let word_break = ch.is_whitespace()
            || (ch == '-' && !self.after_colon && self.flow_depth == 0 && self.token.is_empty());
        if !word_break {
            return None;
        }
        if self.after_colon && !self.scalar.is_empty() {
            if finish_scalar(&self.scalar, self.key_was_uses) {
                *present = true;
            }
            if self.flow_depth == 0 {
                return Some(true);
            }
            self.scalar.clear();
            self.key_was_uses = false;
            self.after_colon = false;
        } else if !self.after_colon {
            self.token.clear();
        }
        Some(false)
    }

    fn finish_key(&mut self) {
        let in_input = self.in_flow_input();
        self.value_key_is_input = !in_input && (self.token == "with" || self.token == "env");
        self.key_was_uses = self.token == "uses" && !in_input;
        self.token.clear();
        self.scalar.clear();
        self.after_colon = true;
    }
}
