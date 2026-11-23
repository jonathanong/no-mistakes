use super::{Lexer, Word};
use crate::codebase::shell_words::{split_quoted_words, Whitespace};

impl<'a> Lexer<'a> {
    pub(super) fn redirection(&mut self) -> Option<Word<'a>> {
        if self.take("<<<") {
            return self.redirect_word();
        }
        if self.take("<<") {
            let strip_tabs = self.take("-");
            let word = self.redirect_word()?;
            let delimiter = if matches!(word.raw, "''" | "\"\"") {
                String::new()
            } else {
                split_quoted_words(word.raw, Whitespace::Ascii)?
                    .into_iter()
                    .next()?
            };
            self.heredocs.push((delimiter, strip_tabs));
            return None;
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
        for (delimiter, strip_tabs) in std::mem::take(&mut self.heredocs) {
            while self.current().is_some() {
                let rest = self.rest();
                let (line, width) = rest
                    .split_once('\n')
                    .map_or((rest, rest.len()), |(line, _)| (line, line.len() + 1));
                self.offset += width;
                self.line += 1;
                let line = line.strip_suffix('\r').unwrap_or(line);
                let line = if strip_tabs {
                    line.trim_start_matches('\t')
                } else {
                    line
                };
                if line == delimiter {
                    break;
                }
            }
        }
    }
}
