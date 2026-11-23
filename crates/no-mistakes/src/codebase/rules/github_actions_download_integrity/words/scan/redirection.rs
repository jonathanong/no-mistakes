use super::{flush_command, heredoc, Scan};

impl Scan<'_> {
    pub(super) fn redirection(&mut self) -> bool {
        let remaining = &self.chars[self.index..];
        if !matches!(remaining.first(), Some('<' | '>' | '&')) || self.escaped() {
            return false;
        }
        if let Some(opener) = heredoc::look(self.chars, self.index) {
            self.flush_before_redirect();
            match opener {
                heredoc::Look::Herestring { index } => {
                    self.redirect_operand = true;
                    self.index = index;
                }
                heredoc::Look::Document { index, opener } => {
                    flush_command(&mut self.current, &mut self.commands);
                    self.current.anchor = self.index + 1;
                    self.heredoc_ranges.push(self.index..index);
                    self.pending = Some(opener);
                    self.index = index;
                }
            }
            return true;
        }
        let length = match remaining {
            ['&', '>', '>', ..] => 3,
            ['&', '>', ..] | ['>', '>' | '&' | '|', ..] | ['<', '>' | '&', ..] => 2,
            ['<' | '>', ..] => 1,
            _ => return false,
        };
        if remaining[0] == '&' {
            self.flush_word();
        } else {
            self.flush_before_redirect();
        }
        self.redirect_operand = true;
        self.index += length;
        true
    }

    fn flush_before_redirect(&mut self) {
        if !self.redirect_operand
            && !self.word_quoted
            && self.word.bytes().all(|byte| byte.is_ascii_digit())
        {
            self.discard_word();
        } else {
            self.flush_word();
        }
    }
}
