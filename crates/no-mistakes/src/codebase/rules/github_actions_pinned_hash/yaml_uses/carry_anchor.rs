use super::Carry;

impl Carry {
    /// An anchor or alias sits between `uses:` and the scalar, or a `*name`
    /// is the whole step. Returns true when this character was consumed.
    pub(super) fn consume_anchor(&mut self, ch: char, hits: &mut usize) -> bool {
        if self.skip_anchor {
            return self.finish_anchor_char(ch, hits);
        }
        if self.after_colon && self.scalar.is_empty() && matches!(ch, '&' | '*') {
            self.start_anchor(ch == '*', ch == '&');
            return true;
        }
        if ch == '&' && self.token.is_empty() && self.scalar.is_empty() {
            self.start_anchor(false, true);
            return true;
        }
        if ch == '*' && !self.after_colon && self.token.is_empty() && self.place.item_is_step() {
            self.start_anchor(false, false);
            self.step_alias = true;
            return true;
        }
        false
    }

    pub(super) fn note_anchor_uses(&mut self) {
        let in_flow = self.anchor_depth > 0 && self.anchor_depth == self.flow_depth;
        let in_block = self.block_anchor && self.flow_depth == 0;
        if self.token == "uses" && (in_flow || in_block) && !self.anchor_name.is_empty() {
            self.remember_anchor();
        }
    }

    pub(super) fn arm_anchor_mapping(&mut self, sequence: bool) {
        if !self.anchor_pending {
            return;
        }
        self.anchor_pending = false;
        if sequence {
            self.anchor_name.clear();
            self.anchor_depth = 0;
        } else {
            self.anchor_depth = self.flow_depth;
        }
    }

    pub(super) fn clear_anchor_watch(&mut self) {
        self.anchor_pending = false;
        self.anchor_depth = 0;
        self.anchor_name.clear();
        self.block_anchor = false;
    }

    pub(super) fn reset_anchor_state(&mut self) {
        self.clear_anchor_watch();
        self.anchors_with_uses.clear();
    }

    pub(super) fn anchor_has_uses(&self) -> bool {
        self.anchors_with_uses
            .iter()
            .any(|name| name == &self.alias_name)
    }

    /// A later line at the anchor's indent is a new mapping.
    pub(super) fn end_block_anchor(&mut self, line: &str) {
        if !self.block_anchor {
            return;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return;
        }
        let indent = line.chars().take_while(|ch| ch.is_whitespace()).count();
        if indent <= self.anchor_indent {
            self.block_anchor = false;
            self.anchor_name.clear();
        }
    }

    fn start_anchor(&mut self, alias: bool, reading: bool) {
        self.alias = alias;
        self.skip_anchor = true;
        self.reading_anchor = reading;
        self.anchor_name.clear();
        self.alias_name.clear();
        if reading {
            self.anchor_indent = self.place.line_indent();
        }
    }

    fn finish_anchor_char(&mut self, ch: char, hits: &mut usize) -> bool {
        let boundary = ch.is_whitespace() || matches!(ch, '{' | '}' | ']' | ',' | ':');
        if !boundary {
            if self.reading_anchor {
                self.anchor_name.push(ch);
            }
            if self.step_alias {
                self.alias_name.push(ch);
            }
            return true;
        }
        let was_alias = self.alias && self.key_was_uses;
        let step_alias = self.step_alias && self.anchor_has_uses();
        self.skip_anchor = false;
        self.alias = false;
        self.step_alias = false;
        self.reading_anchor = false;
        if was_alias || step_alias {
            *hits += 1;
            self.key_was_uses = false;
        }
        if !self.anchor_name.is_empty() && (ch == '{' || ch.is_whitespace()) {
            self.anchor_pending = true;
        } else if ch != '{' {
            self.anchor_name.clear();
        }
        ch.is_whitespace()
    }

    fn remember_anchor(&mut self) {
        let name = self.anchor_name.clone();
        if !self.anchors_with_uses.iter().any(|have| have == &name) {
            self.anchors_with_uses.push(name);
        }
    }
}
