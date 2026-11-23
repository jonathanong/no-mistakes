use super::Lexer;

impl Lexer<'_> {
    pub(super) fn skip_group(&mut self, end: u8) -> bool {
        let mut ends = vec![end];
        while let (Some(byte), Some(&end)) = (self.current(), ends.last()) {
            if byte == end {
                self.advance();
                ends.pop();
                continue;
            }
            if end == b'\'' {
                self.advance();
                continue;
            }
            if byte == b'\\' {
                self.escape();
                continue;
            }
            if let Some(close) = self.take_expansion_start() {
                ends.push(close);
                continue;
            }
            if end != b'"' && end != b'`' {
                match byte {
                    b'(' => ends.push(b')'),
                    b'\'' | b'"' | b'`' => ends.push(byte),
                    _ => {}
                }
            }
            self.advance();
        }
        ends.is_empty()
    }

    pub(super) fn take_expansion_start(&mut self) -> Option<u8> {
        if self.take("${") {
            Some(b'}')
        } else if self.take("$(") {
            Some(b')')
        } else {
            None
        }
    }

    pub(super) fn escape(&mut self) {
        if self.continuation() {
            return;
        }
        self.advance();
        if let Some(ch) = self.rest().chars().next() {
            self.offset += ch.len_utf8();
        }
    }
}
