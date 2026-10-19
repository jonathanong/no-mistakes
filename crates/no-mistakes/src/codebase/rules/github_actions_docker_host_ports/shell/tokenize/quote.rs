use super::Tok;

impl<'a> Tok<'a> {
    pub(super) fn quoted_single(&mut self) -> bool {
        if self.quote != Some('\'') {
            return false;
        }
        let ch = self.ch();
        if ch == '\n' {
            self.line += 1;
        }
        if ch == '\'' {
            self.quote = None;
        } else {
            self.word.push(ch);
        }
        self.index += 1;
        true
    }

    pub(super) fn inside_backtick(&mut self) -> bool {
        if !self.backtick {
            return false;
        }
        self.command.dynamic = true;
        if self.ch() == '\n' {
            self.line += 1;
        }
        if self.ch() == '`' {
            self.backtick = false;
        }
        self.index += 1;
        true
    }

    pub(super) fn inside_subst(&mut self) -> bool {
        if self.subst <= 0 {
            return false;
        }
        self.command.dynamic = true;
        let ch = self.ch();
        if ch == '\n' {
            self.line += 1;
        }
        if ch == '(' {
            self.subst += 1;
        } else if ch == ')' {
            self.subst -= 1;
        }
        self.index += 1;
        true
    }

    pub(super) fn quoted_double(&mut self) -> bool {
        if self.quote != Some('"') {
            return false;
        }
        if self.escaped_quote() || self.double_newline() || self.double_end() {
            return true;
        }
        self.word.push(self.ch());
        self.index += 1;
        true
    }

    fn escaped_quote(&mut self) -> bool {
        if self.ch() != '\\' || self.index + 1 >= self.chars.len() {
            return false;
        }
        let next = self.chars[self.index + 1];
        if next == '\n' {
            self.line += 1;
        } else {
            self.word.push(next);
        }
        self.index += 2;
        true
    }

    fn double_newline(&mut self) -> bool {
        if self.ch() != '\n' {
            return false;
        }
        self.line += 1;
        self.word.push('\n');
        self.index += 1;
        true
    }

    fn double_end(&mut self) -> bool {
        let ch = self.ch();
        if ch == '"' {
            self.quote = None;
            self.index += 1;
            return true;
        }
        if ch == '`' {
            self.command.dynamic = true;
            self.backtick = true;
            self.index += 1;
            return true;
        }
        if ch == '$' && self.chars.get(self.index + 1) == Some(&'(') {
            self.command.dynamic = true;
            self.subst = 1;
            self.index += 2;
            return true;
        }
        false
    }
}
