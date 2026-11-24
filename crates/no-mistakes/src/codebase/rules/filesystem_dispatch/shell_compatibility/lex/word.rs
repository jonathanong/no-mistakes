use super::Lexer;

pub(in super::super) struct Word<'a> {
    pub raw: &'a str,
    pub line: usize,
    pub refs: Vec<(&'a str, usize)>,
    pub array_literal: Option<bool>,
    pub redirect: bool,
    dynamic: bool,
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
                b'$' if !double_quoted && self.take("$'") => self.skip_ansi_quote(),
                b'$' => self.expansion(&mut word, double_quoted),
                b'`' => {
                    word.dynamic |= !double_quoted;
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
                    self.skip_literal_array_index();
                    let item = self.word();
                    literal |= !item.dynamic;
                    references.extend(item.refs);
                }
            }
        }
        literal
    }

    fn skip_literal_array_index(&mut self) {
        if let Some(index) = self.rest().strip_prefix('[') {
            let suffix = index.trim_start_matches(|ch: char| ch.is_ascii_digit());
            if suffix.len() < index.len() && suffix.starts_with("]=") {
                self.offset += 3 + index.len() - suffix.len();
            }
        }
    }

    fn skip_ansi_quote(&mut self) {
        while let Some(byte) = self.current() {
            match byte {
                b'\\' => self.escape(),
                b'\'' => {
                    self.advance();
                    break;
                }
                _ => self.advance(),
            }
        }
    }

    // A double-quoted expansion is one word unless it splices `$@`, `${name[@]}` or `${!prefix@}`.
    fn expansion(&mut self, word: &mut Word<'a>, double_quoted: bool) {
        word.dynamic |= !double_quoted || self.rest().starts_with("$@");
        if let Some(end) = self.take_expansion_start() {
            let start = self.offset;
            let line = self.line;
            if self.skip_group(end) && end == b'}' && double_quoted {
                let body = &self.source[start..self.offset - 1];
                word.dynamic |= body.starts_with('@')
                    || body.contains("[@]")
                    || (body.starts_with('!') && body.ends_with('@'));
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
