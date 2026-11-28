use super::super::expansion::{take_expansion, ANSI_C};
use super::super::{flush_word, push_char};
use super::Scan;

impl Scan<'_> {
    pub(super) fn quoted(&mut self, ch: char) -> bool {
        let Some(quote) = self.quoting else {
            return false;
        };
        if quote == '"' && self.expansion(ch) {
            return true;
        }
        if quote == ANSI_C && ch == '\\' && self.index + 1 < self.chars.len() {
            for escaped in [ch, self.chars[self.index + 1]] {
                push_char(&mut self.word, &mut self.word_line, self.line, escaped);
            }
            if self.chars[self.index + 1] == '\n' {
                self.line += 1;
            }
            self.index += 2;
            return true;
        }
        if ch == quote || (quote == ANSI_C && ch == '\'') {
            self.quoting = None;
        } else {
            push_char(&mut self.word, &mut self.word_line, self.line, ch);
            if ch == '\n' {
                self.line += 1;
            }
        }
        self.index += 1;
        true
    }

    pub(super) fn start_quote(&mut self, ch: char) -> bool {
        if ch != '\'' && ch != '"' {
            return false;
        }
        if self.word.is_empty() && !self.word_quoted {
            self.word_line = self.line;
        }
        let ansi_c = ch == '\'' && self.index > 0 && self.chars[self.index - 1] == '$';
        self.quoting = Some(if ansi_c { ANSI_C } else { ch });
        self.word_quoted = true;
        self.index += 1;
        true
    }

    pub(super) fn expansion(&mut self, ch: char) -> bool {
        if self.escaped() {
            return false;
        }
        if self.quoting.is_none() && matches!(ch, '$' | '`') {
            self.word_expansion_may_split = true;
        }
        let reference = (ch == '$')
            .then(|| quoted_reference_len(&self.chars[self.index + 1..]))
            .flatten()
            .map(|len| (self.word.len(), len));
        if self.quoting.is_none() {
            self.word_bare_expansion = self.word_bare_expansion.or(reference);
        } else {
            self.word_quoted_expansion = self.word_quoted_expansion.or(reference);
        }
        if ch == '`' {
            self.current.dynamic = true;
            if let Some((start, start_line)) = self.backtick_start.take() {
                let body: String = self.chars[start..self.index].iter().collect();
                self.substitutions.push((start_line, body));
            } else {
                self.backtick_start = Some((self.index + 1, self.line));
            }
            self.in_backticks = !self.in_backticks;
        }
        if ch != '$' || !matches!(self.chars.get(self.index + 1), Some(&'(' | &'{')) {
            return false;
        }
        let open = self.chars[self.index + 1];
        self.current.dynamic |= open == '(' || self.quoting != Some('"') || reference.is_none();
        if self.word.is_empty() {
            self.word_line = self.line;
        }
        let body_start = self.index + 2;
        let body_line = self.line;
        let word_at = self.word.len();
        let (next, terminated) =
            take_expansion(self.chars, self.index, &mut self.word, &mut self.line);
        if self.quoting.is_some() {
            let len = self.word.len() - word_at;
            self.word_quoted_expansion = self.word_quoted_expansion.or(Some((word_at, len)));
        }
        if !terminated {
            self.unterminated_substitution = true;
        } else if open == '(' && self.chars.get(body_start) != Some(&'(') {
            let body: String = self.chars[body_start..next - 1].iter().collect();
            self.substitutions.push((body_line, body));
        }
        self.index = next;
        true
    }

    pub(super) fn escaped(&self) -> bool {
        self.escaped_at(self.index)
    }

    pub(super) fn escaped_at(&self, index: usize) -> bool {
        self.chars[..index]
            .iter()
            .rev()
            .take_while(|character| **character == '\\')
            .count()
            % 2
            != 0
    }

    pub(super) fn flush_word(&mut self) {
        let quoted = std::mem::take(&mut self.word_quoted);
        if self.redirect_operand && (quoted || !self.word.is_empty()) {
            self.redirect_operand = false;
            self.discard_word();
            return;
        }
        flush_word(
            &mut self.word,
            &mut self.current,
            self.word_line,
            quoted,
            &mut self.word_expansion_may_split,
            &mut self.word_quoted_expansion,
            &mut self.word_bare_expansion,
        );
    }
    pub(super) fn discard_word(&mut self) {
        self.word.clear();
        self.word_expansion_may_split = false;
        self.word_quoted_expansion = None;
        self.word_bare_expansion = None;
    }
}

fn quoted_reference_len(after_dollar: &[char]) -> Option<usize> {
    let braced = after_dollar.first() == Some(&'{');
    let name = &after_dollar[usize::from(braced)..];
    let len = name
        .iter()
        .take_while(|character| character.is_ascii_alphanumeric() || **character == '_')
        .count();
    if len == 0 || (braced && name.get(len) != Some(&'}')) {
        return None;
    }
    Some(len + if braced { 3 } else { 1 })
}
