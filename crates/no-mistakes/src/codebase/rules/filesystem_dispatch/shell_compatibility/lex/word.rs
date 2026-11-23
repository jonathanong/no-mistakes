use super::Lexer;

pub(in super::super) struct Word<'a> {
    pub raw: &'a str,
    pub line: usize,
    pub refs: Vec<(&'a str, usize)>,
    pub array_literal: Option<bool>,
    pub redirect: bool,
    dynamic: bool,
}

impl Word<'_> {
    pub fn is_literal(&self) -> bool {
        !self.dynamic
    }
}

impl<'a> Lexer<'a> {
    pub(super) fn word(&mut self) -> Word<'a> {
        let start = self.offset;
        let mut word = Word {
            raw: "",
            line: self.line,
            refs: Vec::new(),
            array_literal: None,
            redirect: false,
            dynamic: false,
        };
        let mut double_quoted = false;
        while let Some(byte) = self.current() {
            if !double_quoted && is_boundary(byte) {
                let prefix = &self.source[start..self.offset];
                if byte == b'(' && prefix.ends_with('=') && assignment(prefix).is_some() {
                    self.advance();
                    word.array_literal = Some(self.array_elements(&mut word.refs));
                }
                break;
            }
            match byte {
                b'\\' => self.escape(),
                b'\'' if !double_quoted => {
                    self.advance();
                    self.skip_group(b'\'');
                }
                b'"' => {
                    double_quoted = !double_quoted;
                    self.advance();
                }
                b'$' => self.expansion(&mut word, double_quoted),
                b'`' => {
                    word.dynamic = true;
                    self.advance();
                    self.skip_group(b'`');
                }
                b'*' | b'?' | b'[' if !double_quoted => {
                    word.dynamic = true;
                    self.advance();
                }
                _ => self.advance(),
            }
        }
        word.raw = &self.source[start..self.offset];
        word
    }

    fn array_elements(&mut self, references: &mut Vec<(&'a str, usize)>) -> bool {
        let mut literal = false;
        while let Some(byte) = self.current() {
            if self.continuation() {
                continue;
            }
            match byte {
                b')' => {
                    self.advance();
                    return literal;
                }
                b'#' => self.skip_comment(),
                byte if byte.is_ascii_whitespace() => self.advance(),
                byte if is_boundary(byte) => {
                    self.skip_group(b')');
                    return literal;
                }
                _ => {
                    let item = self.word();
                    literal |= !item.dynamic;
                    references.extend(item.refs);
                }
            }
        }
        literal
    }

    fn expansion(&mut self, word: &mut Word<'a>, double_quoted: bool) {
        word.dynamic = true;
        if let Some(end) = self.take_expansion_start() {
            let start = self.offset;
            let line = self.line;
            if self.skip_group(end) && end == b'}' && double_quoted {
                let body = &self.source[start..self.offset - 1];
                if let Some(name) = body
                    .strip_suffix("[@]")
                    .or_else(|| body.strip_suffix("[*]"))
                {
                    if identifier(name) {
                        word.refs.push((name, line));
                    }
                }
            }
        } else {
            self.advance();
        }
    }
}

fn is_boundary(byte: u8) -> bool {
    byte.is_ascii_whitespace() || matches!(byte, b'(' | b')' | b';' | b'|' | b'&' | b'<' | b'>')
}

pub(in super::super) fn identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|first| first == b'_' || first.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

pub(in super::super) fn assignment(word: &str) -> Option<(&str, bool)> {
    let (lhs, _) = word.split_once('=')?;
    let append = lhs.ends_with('+');
    let name = lhs.strip_suffix('+').unwrap_or(lhs);
    identifier(name).then_some((name, append))
}
