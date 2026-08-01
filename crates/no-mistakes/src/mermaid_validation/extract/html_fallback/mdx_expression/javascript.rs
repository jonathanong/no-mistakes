use super::{Literal, MdxExpressionScanner};

#[path = "javascript/token.rs"]
mod token;
use token::{allows_following_regex, identifier_end, keyword_allows_regex, token_end};

pub(super) enum ByteAction {
    Advance(usize),
    Break,
}

impl MdxExpressionScanner {
    pub(super) fn javascript_token_end(&mut self, line: &[u8], start: usize) -> Option<usize> {
        if !matches!(self.literal, Literal::None) || !self.is_inside_expression() {
            return None;
        }
        let byte = line[start];
        if let Some(end) = identifier_end(line, start) {
            let token = &line[start..end];
            self.can_start_regex = keyword_allows_regex(token);
            if self.esm {
                match token {
                    b"export" if self.last_js_code_byte.is_none() => {
                        self.esm_export_prefix = true;
                        self.esm_value_pending = false;
                    }
                    b"default" if self.esm_export_prefix => {
                        self.esm_export_prefix = false;
                        self.esm_value_pending = true;
                    }
                    _ => {
                        self.esm_export_prefix = false;
                        self.esm_value_pending = false;
                    }
                }
                self.last_js_code_byte = line.get(end - 1).copied();
            }
            return Some(end);
        }
        if byte.is_ascii_digit() {
            self.can_start_regex = false;
            let end = token_end(line, start);
            if self.esm {
                self.last_js_code_byte = line.get(end - 1).copied();
                self.esm_export_prefix = false;
                self.esm_value_pending = false;
            }
            return Some(end);
        }
        None
    }

    pub(super) fn observe_javascript_byte(&mut self, byte: u8, next: Option<u8>) -> ByteAction {
        match self.literal {
            Literal::SingleQuoted => self.observe_quoted(byte, b'\''),
            Literal::DoubleQuoted => self.observe_quoted(byte, b'"'),
            Literal::Template => self.observe_quoted(byte, b'`'),
            Literal::Regex => self.observe_regex(byte),
            Literal::BlockComment => return self.observe_block_comment(byte, next),
            Literal::None => return self.observe_code_byte(byte, next),
        }
        ByteAction::Advance(0)
    }

    fn observe_block_comment(&mut self, byte: u8, next: Option<u8>) -> ByteAction {
        if byte == b'*' && next == Some(b'/') {
            self.literal = Literal::None;
            ByteAction::Advance(1)
        } else {
            ByteAction::Advance(0)
        }
    }

    fn observe_code_byte(&mut self, byte: u8, next: Option<u8>) -> ByteAction {
        let active = self.is_inside_expression();
        if self.esm
            && !byte.is_ascii_whitespace()
            && !matches!((byte, next), (b'/', Some(b'/' | b'*')))
        {
            self.last_js_code_byte = Some(byte);
            self.esm_export_prefix = false;
            self.esm_value_pending = false;
        }
        match (byte, next) {
            (b'/', Some(b'/')) if active => return ByteAction::Break,
            (b'/', Some(b'*')) if active => {
                self.literal = Literal::BlockComment;
                return ByteAction::Advance(1);
            }
            (b'/', _) if active && self.can_start_regex => {
                self.literal = Literal::Regex;
                self.regex_character_class = false;
            }
            (b'/', _) if active => self.can_start_regex = true,
            (b'\'', _) if active => self.literal = Literal::SingleQuoted,
            (b'"', _) if active => self.literal = Literal::DoubleQuoted,
            (b'`', _) if active => self.literal = Literal::Template,
            (b'{', _) => self.open_brace(),
            (b'}', _) if self.depth > 0 => self.close_brace(),
            (b'(', _) if self.esm => self.open_paren(),
            (b')', _) if self.esm && self.paren_depth > 0 => self.close_paren(),
            (b'[', _) if self.esm => self.open_bracket(),
            (b']', _) if self.esm && self.bracket_depth > 0 => self.close_bracket(),
            (b'+' | b'-', Some(next)) if active && next == byte => {
                self.can_start_regex = false;
                return ByteAction::Advance(1);
            }
            (b')' | b']' | b'.', _) if active => self.can_start_regex = false,
            (byte, _) if active && allows_following_regex(byte) => self.can_start_regex = true,
            _ => {}
        }
        ByteAction::Advance(0)
    }

    fn open_brace(&mut self) {
        self.depth += 1;
        self.can_start_regex = true;
    }

    fn close_brace(&mut self) {
        self.depth -= 1;
        self.can_start_regex = false;
        self.resume_jsx_after_expression();
    }

    fn open_paren(&mut self) {
        self.paren_depth += 1;
        self.can_start_regex = true;
    }

    fn close_paren(&mut self) {
        self.paren_depth -= 1;
        self.can_start_regex = false;
    }

    fn open_bracket(&mut self) {
        self.bracket_depth += 1;
        self.can_start_regex = true;
    }

    fn close_bracket(&mut self) {
        self.bracket_depth -= 1;
        self.can_start_regex = false;
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
            return;
        }
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
