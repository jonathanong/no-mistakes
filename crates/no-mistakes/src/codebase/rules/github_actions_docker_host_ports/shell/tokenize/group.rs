use super::commands;
use super::Tok;
use crate::codebase::shell_heredoc::{self, CarriageReturn, Document, Operator};

impl<'a> Tok<'a> {
    pub(super) fn capture_backtick(&mut self) -> bool {
        if self.ch() != '`' {
            return false;
        }
        self.mark_dynamic();
        self.index += 1;
        let start = self.index;
        let start_line = self.line;
        while self.index < self.chars.len() && self.chars[self.index] != '`' {
            if self.chars[self.index] == '\n' {
                self.line += 1;
            }
            self.index += 1;
        }
        if self.index >= self.chars.len() {
            // Unclosed: the word is already dynamic. Keep literals parsed before it.
            return true;
        }
        let body = self.slice(start, self.index);
        self.index += 1;
        self.absorb(&body, start_line);
        true
    }

    pub(super) fn capture_dollar(&mut self) -> bool {
        if self.ch() != '$' || self.chars.get(self.index + 1) != Some(&'(') {
            return false;
        }
        self.capture_balanced(2)
    }

    pub(super) fn capture_process(&mut self) -> bool {
        let ch = self.ch();
        if self.open || (ch != '<' && ch != '>') || self.chars.get(self.index + 1) != Some(&'(') {
            return false;
        }
        self.capture_balanced(2)
    }

    pub(super) fn skip_heredoc(&mut self) -> bool {
        let Some(operator) = shell_heredoc::operator(&self.chars[self.index..], false) else {
            return false;
        };
        if matches!(operator, Operator::Document { .. }) && self.arithmetic_shift() {
            return false;
        }
        self.flush_word();
        self.index += operator.width();
        if let Operator::Document { strip_tabs } = operator {
            let (delimiter, width) = shell_heredoc::char_delimiter(
                &self.chars[self.index..],
                shell_heredoc::DelimiterStyle::StripQuotes,
            );
            self.index += width;
            self.pending_heredocs.push(Document { delimiter, strip_tabs });
        }
        true
    }

    fn capture_balanced(&mut self, opener: usize) -> bool {
        self.mark_dynamic();
        self.index += opener;
        let start = self.index;
        let start_line = self.line;
        if self.skip_until_close() {
            let body = self.slice(start, self.index.saturating_sub(1));
            self.absorb(&body, start_line);
        }
        true
    }

    // `(( n << 1 ))` and `let` arguments are shifts. Only the command's first
    // word counts, so a later quoted `"(("` does not hide a real heredoc.
    fn arithmetic_shift(&self) -> bool {
        match self.command.words.first().map(|word| word.text.as_str()) {
            Some("let") => true,
            Some(text) => text.starts_with("(("),
            None => self.word.starts_with("(("),
        }
    }

    fn mark_dynamic(&mut self) {
        self.mark_open();
        self.word_dynamic = true;
    }

    fn slice(&self, start: usize, end: usize) -> String {
        self.chars[start..end].iter().collect()
    }

    fn absorb(&mut self, body: &str, start_line: usize) {
        for mut command in commands(body) {
            for word in &mut command.words {
                word.line = word.line.saturating_add(start_line.saturating_sub(1));
            }
            self.commands.push(command);
        }
    }

    fn skip_until_close(&mut self) -> bool {
        let mut depth = 1i32;
        let mut quote = None;
        while self.index < self.chars.len() {
            let ch = self.chars[self.index];
            if ch == '\n' {
                self.line += 1;
            }
            if let Some(active) = quote {
                if ch == active {
                    quote = None;
                }
                self.index += 1;
                continue;
            }
            if ch == '\'' || ch == '"' {
                quote = Some(ch);
                self.index += 1;
                continue;
            }
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
                self.index += 1;
                if depth == 0 {
                    return true;
                }
                continue;
            }
            self.index += 1;
        }
        false
    }

    pub(super) fn skip_pending_bodies(&mut self) {
        for document in self.pending_heredocs.drain(..) {
            let body = shell_heredoc::skip_body(
                self.chars,
                self.index,
                document.delimiter.chars(),
                document.strip_tabs,
                CarriageReturn::Keep,
            );
            self.index = body.end;
            self.line += body.newlines;
            if !body.terminated {
                break;
            }
        }
    }
}
