use super::super::{flush_word, push_char, take_expansion};
use super::Scan;

impl Scan<'_> {
    pub(super) fn quoted(&mut self, ch: char) -> bool {
        let Some(quote) = self.quoting else {
            return false;
        };
        if quote == '"' && self.expansion(ch) {
            return true;
        }
        if ch == quote {
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
        self.quoting = Some(ch);
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
        if self.quoting == Some('"') && ch == '$' {
            self.word_quoted_expansion = quoted_reference_len(&self.chars[self.index + 1..])
                .map(|len| (self.word.len(), len));
        }
        if ch == '`' {
            self.current.dynamic = true;
        }
        if ch != '$' || !matches!(self.chars.get(self.index + 1), Some(&'(' | &'{')) {
            return false;
        }
        self.current.dynamic = true;
        if self.word.is_empty() {
            self.word_line = self.line;
        }
        self.index = take_expansion(self.chars, self.index, &mut self.word, &mut self.line);
        true
    }

    pub(super) fn escaped(&self) -> bool {
        self.chars[..self.index]
            .iter()
            .rev()
            .take_while(|character| **character == '\\')
            .count()
            % 2
            != 0
    }

    pub(super) fn flush_word(&mut self) {
        flush_word(
            &mut self.word,
            &mut self.current,
            self.word_line,
            &mut self.word_expansion_may_split,
            &mut self.word_quoted_expansion,
        );
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
