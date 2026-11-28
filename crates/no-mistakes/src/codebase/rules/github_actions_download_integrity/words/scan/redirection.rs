use super::{flush_command, HeredocBody, Scan};
use crate::codebase::command_lex::heredoc::{self as shell_heredoc, CarriageReturn, Document, Operator};

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
                    let (delimiter, width, quoted) =
                        shell_heredoc::char_delimiter_quoted(&self.chars[operand..]);
                    let end = operand + width;
                    flush_command(&mut self.current, &mut self.commands);
                    self.current.anchor = self.index + 1;
                    self.heredoc_ranges.push(self.index..end);
                    self.pending.push((Document { delimiter, strip_tabs }, quoted));
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

    pub(super) fn skip_heredocs(&mut self) {
        let start = self.index + 1;
        self.index = start;
        self.line += 1;
        for (opener, quoted) in self.pending.drain(..) {
            let body_start = self.index;
            let body = shell_heredoc::skip_body(
                self.chars,
                self.index,
                opener.delimiter.chars(),
                opener.strip_tabs,
                CarriageReturn::BeforeNewline,
            );
            self.index = body.end;
            self.line += body.newlines;
            self.heredoc_bodies.push(HeredocBody {
                range: body_start..body.end,
                quoted,
            });
            if !body.terminated {
                self.unterminated = true;
                self.stop = true;
                break;
            }
        }
        self.heredoc_ranges.push(start..self.index);
        self.current.anchor = self.index;
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
