use super::cursor::{is_ident_char, is_ident_start, Cursor};

impl<'a> Cursor<'a> {
    pub(super) fn take_qualified(&mut self) -> Result<String, String> {
        let mut name = self.take_ident()?;
        while self.eat_char('.') {
            name = self.take_ident()?;
        }
        Ok(name)
    }

    pub(super) fn take_ident(&mut self) -> Result<String, String> {
        self.skip_ws();
        if self.rest().starts_with('"') {
            return self.take_quoted();
        }
        let rest = self.rest();
        let end = rest
            .char_indices()
            .find(|(_, character)| !is_ident_char(*character))
            .map(|(index, _)| index)
            .unwrap_or(rest.len());
        if end == 0 || !is_ident_start(rest.chars().next().unwrap()) {
            return Err("expected identifier".to_string());
        }
        let name = rest[..end].to_string();
        self.index += end;
        Ok(name)
    }

    fn take_quoted(&mut self) -> Result<String, String> {
        self.index += 1;
        let mut name = String::new();
        while let Some(character) = self.rest().chars().next() {
            self.index += character.len_utf8();
            if character == '"' {
                if self.rest().starts_with('"') {
                    name.push('"');
                    self.index += 1;
                    continue;
                }
                return Ok(name);
            }
            name.push(character);
        }
        Err("unterminated identifier".to_string())
    }

    pub(super) fn take_arguments(&mut self) -> Result<Vec<String>, String> {
        self.expect_char('(')?;
        let mut arguments = Vec::new();
        if self.eat_char(')') {
            return Ok(arguments);
        }
        loop {
            arguments.push(self.take_string()?);
            if self.eat_char(',') {
                continue;
            }
            self.expect_char(')')?;
            break;
        }
        Ok(arguments)
    }

    fn take_string(&mut self) -> Result<String, String> {
        self.skip_ws();
        if !self.eat_char('\'') {
            return Err("expected string".to_string());
        }
        let mut value = String::new();
        while let Some(character) = self.rest().chars().next() {
            self.index += character.len_utf8();
            if character == '\'' {
                if self.rest().starts_with('\'') {
                    value.push('\'');
                    self.index += 1;
                    continue;
                }
                return Ok(value);
            }
            value.push(character);
        }
        Err("unterminated string".to_string())
    }

    pub(super) fn take_paren_inner(&mut self) -> Result<String, String> {
        self.expect_char('(')?;
        let start = self.index;
        let mut depth = 1i32;
        let mut quote: Option<char> = None;
        while let Some(character) = self.rest().chars().next() {
            self.index += character.len_utf8();
            if let Some(open) = quote {
                if character == open {
                    if self.rest().starts_with(open) {
                        self.index += open.len_utf8();
                    } else {
                        quote = None;
                    }
                }
                continue;
            }
            match character {
                '\'' | '"' => quote = Some(character),
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(self.source[start..self.index - 1].to_string());
                    }
                }
                _ => {}
            }
        }
        Err("unbalanced WHEN condition".to_string())
    }
}
