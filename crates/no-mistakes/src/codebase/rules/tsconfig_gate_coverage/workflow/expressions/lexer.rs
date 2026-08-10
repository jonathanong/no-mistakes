#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Token {
    Identifier,
    Boolean,
    Number,
    String,
    Null,
    Bang,
    And,
    Or,
    Comparison,
    Dot,
    Star,
    Comma,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
}

pub(super) fn tokenize(body: &str) -> Option<Vec<Token>> {
    let bytes = body.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            byte if byte.is_ascii_whitespace() => index += 1,
            b'\'' => {
                index = string_end(bytes, index + 1)?;
                tokens.push(Token::String);
            }
            b'0'..=b'9' | b'-' if number_starts(bytes, index) => {
                index = number_end(bytes, index)?;
                tokens.push(Token::Number);
            }
            byte if identifier_start(byte) => {
                let start = index;
                index += 1;
                while index < bytes.len() && identifier_continue(bytes[index]) {
                    index += 1;
                }
                tokens.push(match &body[start..index].to_ascii_lowercase()[..] {
                    "true" | "false" => Token::Boolean,
                    "null" => Token::Null,
                    _ => Token::Identifier,
                });
            }
            b'!' if bytes.get(index + 1) == Some(&b'=') => {
                tokens.push(Token::Comparison);
                index += 2;
            }
            b'!' => {
                tokens.push(Token::Bang);
                index += 1;
            }
            b'=' if bytes.get(index + 1) == Some(&b'=') => {
                tokens.push(Token::Comparison);
                index += 2;
            }
            b'>' | b'<' => {
                tokens.push(Token::Comparison);
                index += usize::from(bytes.get(index + 1) == Some(&b'=')) + 1;
            }
            b'&' if bytes.get(index + 1) == Some(&b'&') => {
                tokens.push(Token::And);
                index += 2;
            }
            b'|' if bytes.get(index + 1) == Some(&b'|') => {
                tokens.push(Token::Or);
                index += 2;
            }
            b'.' => {
                tokens.push(Token::Dot);
                index += 1;
            }
            b'*' => {
                tokens.push(Token::Star);
                index += 1;
            }
            b',' => {
                tokens.push(Token::Comma);
                index += 1;
            }
            b'(' => {
                tokens.push(Token::LeftParen);
                index += 1;
            }
            b')' => {
                tokens.push(Token::RightParen);
                index += 1;
            }
            b'[' => {
                tokens.push(Token::LeftBracket);
                index += 1;
            }
            b']' => {
                tokens.push(Token::RightBracket);
                index += 1;
            }
            _ => return None,
        }
    }
    (!tokens.is_empty()).then_some(tokens)
}

fn string_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    while index < bytes.len() {
        if bytes[index] == b'\'' {
            if bytes.get(index + 1) == Some(&b'\'') {
                index += 2;
            } else {
                return Some(index + 1);
            }
        } else {
            index += 1;
        }
    }
    None
}

fn number_starts(bytes: &[u8], index: usize) -> bool {
    bytes[index].is_ascii_digit()
        || (bytes[index] == b'-' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit))
}

fn number_end(bytes: &[u8], mut index: usize) -> Option<usize> {
    if bytes[index] == b'-' {
        index += 1;
    }
    if bytes.get(index) == Some(&b'0') && matches!(bytes.get(index + 1), Some(b'x' | b'X')) {
        index += 2;
        let start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_hexdigit) {
            index += 1;
        }
        return (index > start).then_some(index);
    }
    let integer_start = index;
    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }
    if index == integer_start {
        return None;
    }
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fractional_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == fractional_start {
            return None;
        }
    }
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let exponent_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == exponent_start {
            return None;
        }
    }
    Some(index)
}

fn identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}
