#[derive(Clone, Copy, Default)]
enum Literal {
    #[default]
    None,
    SingleQuoted,
    DoubleQuoted,
    Template,
    BlockComment,
}

#[derive(Default)]
pub(crate) struct MdxExpressionScanner {
    depth: usize,
    literal: Literal,
    escaped: bool,
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
            match self.literal {
                Literal::SingleQuoted => self.observe_quoted(byte, b'\''),
                Literal::DoubleQuoted => self.observe_quoted(byte, b'"'),
                Literal::Template => self.observe_quoted(byte, b'`'),
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
                    (b'\'', _) if self.depth > 0 => self.literal = Literal::SingleQuoted,
                    (b'"', _) if self.depth > 0 => self.literal = Literal::DoubleQuoted,
                    (b'`', _) if self.depth > 0 => self.literal = Literal::Template,
                    (b'{', _) => self.depth += 1,
                    (b'}', _) if self.depth > 0 => self.depth -= 1,
                    _ => {}
                },
            }
            index += 1;
            if stop_when_closed && self.depth == 0 {
                self.escaped = false;
                return true;
            }
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
        }
    }
}

#[cfg(test)]
#[path = "mdx_expression/tests.rs"]
mod tests;
