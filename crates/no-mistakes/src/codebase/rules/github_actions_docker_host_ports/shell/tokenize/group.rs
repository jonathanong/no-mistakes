use super::commands;
use super::Tok;

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
        let closed = self.index < self.chars.len();
        let body = self.slice(start, self.index);
        if closed {
            self.index += 1;
        } else {
            self.command.dynamic = true;
        }
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
        if self.open || self.ch() != '<' || self.chars.get(self.index + 1) != Some(&'<') {
            return false;
        }
        if self.chars.get(self.index + 2) == Some(&'<') {
            return false;
        }
        self.index += 2;
        let strip_tabs = self.bump_if('-');
        let delim = self.heredoc_delim();
        self.finish_line();
        while self.index < self.chars.len() {
            let mut line = String::new();
            while self.index < self.chars.len() && self.chars[self.index] != '\n' {
                line.push(self.chars[self.index]);
                self.index += 1;
            }
            let compare = if strip_tabs {
                line.trim_start_matches('\t')
            } else {
                line.as_str()
            };
            let done = compare == delim;
            self.finish_line();
            if done {
                break;
            }
        }
        // The terminator ends this command. The next line is a new one.
        self.flush_word();
        self.flush_command();
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
        } else {
            self.command.dynamic = true;
        }
        true
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

    fn bump_if(&mut self, expected: char) -> bool {
        if self.index < self.chars.len() && self.chars[self.index] == expected {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn heredoc_delim(&mut self) -> String {
        let quote = if self.index < self.chars.len() && matches!(self.chars[self.index], '\'' | '"')
        {
            let quote = self.chars[self.index];
            self.index += 1;
            Some(quote)
        } else {
            None
        };
        let mut delim = String::new();
        while self.index < self.chars.len() {
            let ch = self.chars[self.index];
            if quote == Some(ch) {
                self.index += 1;
                break;
            }
            if quote.is_none() && (ch.is_whitespace() || ch == ';') {
                break;
            }
            delim.push(ch);
            self.index += 1;
        }
        delim
    }

    fn finish_line(&mut self) {
        while self.index < self.chars.len() && self.chars[self.index] != '\n' {
            self.index += 1;
        }
        if self.index < self.chars.len() {
            self.index += 1;
            self.line += 1;
        }
    }
}
