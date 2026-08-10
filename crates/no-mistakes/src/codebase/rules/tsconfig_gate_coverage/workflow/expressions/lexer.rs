#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Token {
    Identifier,
    Boolean,
    Number,
    String,
    Null,
    Function(Function),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Function {
    Contains,
    StartsWith,
    EndsWith,
    Format,
    Join,
    ToJson,
    FromJson,
    HashFiles,
    Case,
    Success,
    Failure,
    Always,
    Cancelled,
}

impl Function {
    pub(super) fn accepts_argument_count(self, count: usize) -> bool {
        match self {
            Self::Contains | Self::StartsWith | Self::EndsWith => count == 2,
            // GitHub requires a format string and at least one replacement.
            Self::Format => count >= 2,
            Self::Join => (1..=2).contains(&count),
            Self::ToJson | Self::FromJson => count == 1,
            // `hashFiles` accepts one or more comma-separated patterns.
            Self::HashFiles => count >= 1,
            // `case` takes predicate/value pairs and a final default value.
            Self::Case => count >= 3 && count % 2 == 1,
            Self::Success | Self::Failure | Self::Always | Self::Cancelled => count == 0,
        }
    }
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
            b'0'..=b'9' | b'-' if numbers::starts(bytes, index) => {
                index = numeric_literal_end(bytes, index)?;
                tokens.push(Token::Number);
            }
            byte if identifier_start(byte) => {
                let start = index;
                index += 1;
                while index < bytes.len() && identifier_continue(bytes[index]) {
                    index += 1;
                }
                let identifier = &body[start..index];
                tokens.push(match &identifier.to_ascii_lowercase()[..] {
                    "true" | "false" => Token::Boolean,
                    "null" => Token::Null,
                    _ if follows_call(bytes, index) => {
                        function(identifier).map_or(Token::Identifier, Token::Function)
                    }
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

pub(super) fn numeric_literal_end(bytes: &[u8], index: usize) -> Option<usize> {
    numbers::starts(bytes, index).then(|| numbers::end(bytes, index))?
}

// GitHub's expression-function reference enumerates this closed set and their
// signatures:
// https://docs.github.com/en/actions/reference/workflows-and-actions/expressions#functions
// Status functions are included because `if` conditions use them directly.
fn function(identifier: &str) -> Option<Function> {
    Some(match identifier.to_ascii_lowercase().as_str() {
        "contains" => Function::Contains,
        "startswith" => Function::StartsWith,
        "endswith" => Function::EndsWith,
        "format" => Function::Format,
        "join" => Function::Join,
        "tojson" => Function::ToJson,
        "fromjson" => Function::FromJson,
        "hashfiles" => Function::HashFiles,
        "case" => Function::Case,
        "success" => Function::Success,
        "failure" => Function::Failure,
        "always" => Function::Always,
        "cancelled" => Function::Cancelled,
        _ => return None,
    })
}

fn follows_call(bytes: &[u8], mut index: usize) -> bool {
    while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
        index += 1;
    }
    bytes.get(index) == Some(&b'(')
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

fn identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}
mod numbers;
