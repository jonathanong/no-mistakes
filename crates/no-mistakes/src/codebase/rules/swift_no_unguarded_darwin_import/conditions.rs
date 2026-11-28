use super::imports::{valid_module_name, Proof};

pub(super) fn parse_condition(tail: &str) -> Proof<'_> {
    let text = extract_condition(tail);
    let mut parser = Parser { text, pos: 0 };
    match parser.parse_or() {
        Some(proof) if parser.at_end() => proof,
        _ => Proof::None,
    }
}

fn is_cond_ws(ch: char) -> bool {
    ch.is_whitespace() || ch == '\x00' || ch == '\x0B'
}

fn extract_condition(tail: &str) -> &str {
    let mut end = 0;
    let mut paren_depth = 0usize;
    let mut rest = tail;
    while !rest.is_empty() {
        let (line, after) = rest.split_once('\n').unwrap_or((rest, ""));
        end = line.as_ptr() as usize - tail.as_ptr() as usize + line.len();
        for ch in line.chars() {
            if ch == '(' {
                paren_depth += 1;
            } else if ch == ')' {
                paren_depth = paren_depth.saturating_sub(1);
            }
        }
        let trimmed = line.trim_matches(is_cond_ws);
        let next = after.trim_start_matches(is_cond_ws);
        if paren_depth > 0
            || trimmed.ends_with("&&")
            || trimmed.ends_with("||")
            || next.starts_with("&&")
            || next.starts_with("||")
        {
            rest = after;
        } else {
            break;
        }
    }
    &tail[..end]
}

struct Parser<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn rest(&self) -> &'a str {
        &self.text[self.pos..]
    }

    fn skip_ws(&mut self) {
        self.pos += self.rest().len() - self.rest().trim_start_matches(is_cond_ws).len();
    }

    fn at_end(&mut self) -> bool {
        self.skip_ws();
        self.pos == self.text.len()
    }

    fn eat(&mut self, token: &str) -> bool {
        self.skip_ws();
        if self.rest().starts_with(token) {
            self.pos += token.len();
            true
        } else {
            false
        }
    }

    fn parse_or(&mut self) -> Option<Proof<'a>> {
        let mut lhs = self.parse_and()?;
        while self.eat("||") {
            lhs = lhs.or(self.parse_and()?);
        }
        Some(lhs)
    }

    fn parse_and(&mut self) -> Option<Proof<'a>> {
        let mut lhs = self.parse_unary()?;
        while self.eat("&&") {
            lhs = lhs.and(self.parse_unary()?);
        }
        Some(lhs)
    }

    fn parse_unary(&mut self) -> Option<Proof<'a>> {
        if self.eat("!") {
            self.skip_ws();
            let start = self.pos;
            let _ = self.parse_unary()?;
            let operand = self.text[start..self.pos].trim_matches(is_cond_ws);
            let excludes_linux = operand
                .strip_prefix("os(")
                .and_then(|rest| rest.strip_suffix(')'))
                .is_some_and(|arg| arg.trim_matches(is_cond_ws).trim_matches('`') == "Linux");
            return Some(if excludes_linux { Proof::All } else { Proof::None });
        }
        if self.eat("(") {
            let inner = self.parse_or()?;
            return self.eat(")").then_some(inner);
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Option<Proof<'a>> {
        self.skip_ws();
        let name = self.read_ident()?;
        self.skip_ws();
        if !self.rest().starts_with('(') {
            return Some(Proof::None);
        }
        let args = self.read_paren_args()?;
        Some(match name {
            "canImport" => can_import_proof(args),
            "os" => os_proof(args),
            _ => Proof::None,
        })
    }

    fn read_ident(&mut self) -> Option<&'a str> {
        let rest = self.rest();
        if let Some(after) = rest.strip_prefix('`') {
            let end = after.find('`')?;
            let ident = &after[..end];
            return valid_module_name(ident).then(|| {
                self.pos += end + 2;
                ident
            });
        }
        let mut chars = rest.char_indices();
        let (_, first) = chars.next()?;
        if !(first == '_' || first.is_alphabetic()) {
            return None;
        }
        let end = chars
            .find(|&(_, ch)| !(ch == '_' || ch.is_alphanumeric()))
            .map_or(rest.len(), |(idx, _)| idx);
        self.pos += end;
        Some(&rest[..end])
    }

    fn read_paren_args(&mut self) -> Option<&'a str> {
        let start = self.pos + 1;
        let mut depth = 0usize;
        for (offset, ch) in self.rest().char_indices() {
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
                if depth == 0 {
                    let end = self.pos + offset;
                    self.pos = end + 1;
                    return Some(self.text[start..end].trim_matches(is_cond_ws));
                }
            }
        }
        None
    }
}

fn can_import_proof(args: &str) -> Proof<'_> {
    let module = args
        .split(',')
        .next()
        .unwrap_or("")
        .trim_matches(is_cond_ws)
        .trim_matches('`');
    if valid_module_name(module) {
        Proof::Modules(vec![module])
    } else {
        Proof::None
    }
}

fn os_proof(args: &str) -> Proof<'static> {
    match args.trim_matches('`') {
        "macOS" | "OSX" | "iOS" | "tvOS" | "watchOS" | "visionOS" | "xrOS" | "Windows"
        | "Android" | "FreeBSD" | "OpenBSD" | "PS4" | "Cygwin" | "Haiku" | "WASI"
        | "Emscripten" | "Firmware" | "none" => Proof::All,
        _ => Proof::None,
    }
}
