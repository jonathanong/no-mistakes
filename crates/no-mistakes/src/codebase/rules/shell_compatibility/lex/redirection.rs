use super::{Lexer, Shell, Word};
use crate::codebase::command_lex::heredoc::{
    self as shell_heredoc, CarriageReturn, Document, Operator,
};

impl<'a> Lexer<'a> {
    pub(super) fn is_fd_prefix(&self, word: &str) -> bool {
        let rest = self.rest().as_bytes();
        matches!(rest, [b'<' | b'>', ..])
            && !matches!(rest, [_, b'(', ..])
            && (matches!(self.shell, Shell::Bash) || word.len() == 1)
            && word.bytes().all(|byte| byte.is_ascii_digit())
    }

    pub(super) fn redirection(&mut self) -> Option<Word<'a>> {
        if let Some(operator) = shell_heredoc::operator(self.rest().as_bytes(), false) {
            self.offset += operator.width();
            let word = self.redirect_word()?;
            return match operator {
                Operator::HereString => Some(word),
                Operator::Document { strip_tabs } => {
                    let delimiter = shell_heredoc::unquote(word.raw)?;
                    self.heredocs.push(Document {
                        delimiter,
                        strip_tabs,
                    });
                    None
                }
            };
        }
        if self.take("&>>") || self.take("&>") {
            return self.redirect_word();
        }
        self.advance();
        if self
            .current()
            .is_some_and(|byte| matches!(byte, b'<' | b'>' | b'&' | b'|'))
        {
            self.advance();
        }
        if self.take("(") {
            self.skip_group(b')');
            return None;
        }
        self.redirect_word()
    }

    fn redirect_word(&mut self) -> Option<Word<'a>> {
        while matches!(self.current(), Some(b' ' | b'\t')) || self.continuation() {
            if matches!(self.current(), Some(b' ' | b'\t')) {
                self.advance();
            }
        }
        self.current().filter(|byte| {
            !matches!(byte, b'\n' | b';' | b'|' | b'&' | b'<' | b'>' | b'(' | b')')
        })?;
        let start = self.offset;
        let mut word = self.word();
        if self.offset == start {
            return None;
        }
        word.redirect = true;
        Some(word)
    }

    pub(super) fn skip_heredocs(&mut self) {
        let source = self.source.as_bytes();
        for document in self.heredocs.drain(..) {
            let body = shell_heredoc::skip_body(
                source,
                self.offset,
                document.delimiter.bytes(),
                document.strip_tabs,
                CarriageReturn::StripTrailing,
            );
            self.line += body.newlines
                + usize::from(body.end > self.offset && source[body.end - 1] != b'\n');
            self.offset = body.end;
        }
    }
}
