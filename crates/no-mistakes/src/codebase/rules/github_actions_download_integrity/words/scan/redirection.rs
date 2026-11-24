use super::{flush_command, Scan};
use crate::codebase::shell_heredoc::{self, DelimiterStyle, Document, Operator};

impl Scan<'_> {
    pub(super) fn redirection(&mut self) -> bool {
        let remaining = &self.chars[self.index..];
        if !matches!(remaining.first(), Some('<' | '>' | '&')) || self.escaped() {
            return false;
        }
        if let Some(opener) = shell_heredoc::operator(remaining, false) {
            self.flush_before_redirect();
            let operand = self.index + opener.width();
            match opener {
                Operator::HereString => {
                    self.redirect_operand = true;
                    self.index = operand;
                }
                Operator::Document { strip_tabs } => {
                    let (delimiter, width) = shell_heredoc::char_delimiter(
                        &self.chars[operand..],
                        DelimiterStyle::LeadingQuote,
                    );
                    let end = operand + width;
                    flush_command(&mut self.current, &mut self.commands);
                    self.current.anchor = self.index + 1;
                    self.heredoc_ranges.push(self.index..end);
                    self.pending.push(Document { delimiter, strip_tabs });
                    self.index = end;
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
