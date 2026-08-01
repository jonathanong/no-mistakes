#[derive(Clone, Copy, Default)]
enum Literal {
    #[default]
    None,
    SingleQuoted,
    DoubleQuoted,
    Template,
    Regex,
    BlockComment,
}

#[derive(Default)]
pub(crate) struct MdxExpressionScanner {
    depth: usize,
    literal: Literal,
    escaped: bool,
    regex_character_class: bool,
    can_start_regex: bool,
}

impl MdxExpressionScanner {
    pub(crate) fn is_inside_expression(&self) -> bool {
        self.depth > 0
    }

    pub(super) fn observe_line(&mut self, line: &[u8]) {
        self.observe_line_until_closed(line, false);
    }

    fn observe_line_until_closed(&mut self, line: &[u8], stop_when_closed: bool) -> bool {
        let mut index = 0;
        while index < line.len() {
            let byte = line[index];
            let next = line.get(index + 1).copied();
            if matches!(self.literal, Literal::None) && self.depth > 0 {
                if is_identifier_start(byte) {
                    let end = identifier_end(line, index);
                    self.can_start_regex = keyword_allows_regex(&line[index..end]);
                    index = end;
                    continue;
                }
                if byte.is_ascii_digit() {
                    index = token_end(line, index);
                    self.can_start_regex = false;
                    continue;
                }
            }
            match self.literal {
                Literal::SingleQuoted => self.observe_quoted(byte, b'\''),
                Literal::DoubleQuoted => self.observe_quoted(byte, b'"'),
                Literal::Template => self.observe_quoted(byte, b'`'),
                Literal::Regex => self.observe_regex(byte),
                Literal::BlockComment => {
                    if byte == b'*' && next == Some(b'/') {
                        self.literal = Literal::None;
                        index += 1;
                    }
                }
                Literal::None => match (byte, next) {
                    (b'/', Some(b'/')) if self.depth > 0 => break,
                    (b'/', Some(b'*')) if self.depth > 0 => {
                        self.literal = Literal::BlockComment;
                        index += 1;
                    }
                    (b'/', _) if self.depth > 0 && self.can_start_regex => {
                        self.literal = Literal::Regex;
                        self.regex_character_class = false;
                    }
                    (b'/', _) if self.depth > 0 => self.can_start_regex = true,
                    (b'\'', _) if self.depth > 0 => self.literal = Literal::SingleQuoted,
                    (b'"', _) if self.depth > 0 => self.literal = Literal::DoubleQuoted,
                    (b'`', _) if self.depth > 0 => self.literal = Literal::Template,
                    (b'{', _) => {
                        self.depth += 1;
                        self.can_start_regex = true;
                    }
                    (b'}', _) if self.depth > 0 => {
                        self.depth -= 1;
                        self.can_start_regex = false;
                    }
                    (b'+' | b'-', Some(next)) if self.depth > 0 && next == byte => {
                        self.can_start_regex = false;
                        index += 1;
                    }
                    (b')' | b']' | b'.', _) if self.depth > 0 => {
                        self.can_start_regex = false;
                    }
                    (
                        b'(' | b'[' | b',' | b':' | b';' | b'?' | b'!' | b'=' | b'+' | b'-' | b'*'
                        | b'%' | b'&' | b'|' | b'^' | b'~' | b'<' | b'>',
                        _,
                    ) if self.depth > 0 => self.can_start_regex = true,
                    _ => {}
                },
            }
            index += 1;
            if stop_when_closed && self.depth == 0 {
                self.escaped = false;
                return true;
            }
        }
        if matches!(self.literal, Literal::Regex) {
            self.literal = Literal::None;
            self.regex_character_class = false;
        }
        self.escaped = false;
        false
    }

    pub(crate) fn observe_active_source(&mut self, source: &[u8]) {
        for line in source.split(|byte| matches!(byte, b'\r' | b'\n')) {
            if self.observe_line_until_closed(line, true) {
                break;
            }
        }
    }

    fn observe_quoted(&mut self, byte: u8, delimiter: u8) {
        if self.escaped {
            self.escaped = false;
        } else if byte == b'\\' {
            self.escaped = true;
        } else if byte == delimiter {
            self.literal = Literal::None;
            self.can_start_regex = false;
        }
    }

    fn observe_regex(&mut self, byte: u8) {
        if self.escaped {
            self.escaped = false;
        } else {
            match byte {
                b'\\' => self.escaped = true,
                b'[' => self.regex_character_class = true,
                b']' => self.regex_character_class = false,
                b'/' if !self.regex_character_class => {
                    self.literal = Literal::None;
                    self.can_start_regex = false;
                }
                _ => {}
            }
        }
    }
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$')
}

fn identifier_end(line: &[u8], start: usize) -> usize {
    line[start..]
        .iter()
        .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
        .count()
        + start
}

fn token_end(line: &[u8], start: usize) -> usize {
    line[start..]
        .iter()
        .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_'))
        .count()
        + start
}

fn keyword_allows_regex(identifier: &[u8]) -> bool {
    matches!(
        identifier,
        b"return"
            | b"throw"
            | b"case"
            | b"delete"
            | b"void"
            | b"typeof"
            | b"new"
            | b"in"
            | b"instanceof"
            | b"yield"
            | b"await"
            | b"else"
            | b"do"
    )
}

#[cfg(test)]
#[path = "mdx_expression/tests.rs"]
mod tests;
