//! Preserve E-mode quote continuations through the one source tokenizer pass.
use sqlparser::tokenizer::{Location, Token, TokenWithSpan};
use std::collections::HashMap;

pub(super) struct Prepared {
    pub sql: String,
    continuations: HashMap<Location, String>,
}

impl Prepared {
    pub fn restore(self, tokens: &mut [TokenWithSpan]) {
        for token in tokens {
            if let Some(value) = self.continuations.get(&token.span.start) {
                token.token = Token::SingleQuotedString(value.clone());
            }
        }
    }
}

pub(super) fn prepare(sql: &str) -> Prepared {
    let mut cursor = Cursor {
        sql,
        at: 0,
        location: Location { line: 1, column: 1 },
    };
    let mut output = sql.as_bytes().to_vec();
    let mut continuations = HashMap::new();
    while let Some(character) = cursor.peek() {
        if cursor.starts("--") {
            cursor.line_comment();
        } else if cursor.starts("/*") {
            cursor.block_comment();
        } else if character == '$' && !cursor.identifier_before() && cursor.dollar_body() {
            // Dollar bodies are decoded and prepared only by their own boundary.
        } else if character == '"' || character == '\'' {
            cursor.quoted(character, false, false, &mut output);
        } else if matches!(character, 'e' | 'E')
            && cursor.sql.as_bytes().get(cursor.at + 1) == Some(&b'\'')
            && !cursor.identifier_before()
        {
            cursor.next();
            if cursor.quoted('\'', true, false, &mut output).is_none() {
                continue;
            }
            loop {
                let mut newline = false;
                while cursor
                    .peek()
                    .is_some_and(|character| character.is_ascii_whitespace())
                    || cursor.starts("--")
                {
                    if cursor.starts("--") {
                        newline |= cursor.line_comment();
                    } else {
                        newline |= cursor.next() == Some('\n');
                    }
                }
                if !newline || cursor.peek() != Some('\'') {
                    break;
                }
                let location = cursor.location;
                let start = cursor.at + 1;
                let Some(end) = cursor.quoted('\'', true, true, &mut output) else {
                    break;
                };
                continuations.insert(location, plain_value(&sql[start..end]));
            }
        } else {
            cursor.next();
        }
    }
    Prepared {
        // Only ASCII quote bytes are replaced, so UTF-8 and all locations survive.
        sql: String::from_utf8(output).unwrap(),
        continuations,
    }
}

/// Collapse doubled quotes while retaining escapes for the existing E decoder.
fn plain_value(value: &str) -> String {
    let mut output = String::new();
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        output.push(character);
        if character == '\\' {
            // Closed E-mode strings cannot end in an incomplete escape.
            output.push(characters.next().unwrap());
        } else if character == '\'' && characters.peek() == Some(&'\'') {
            characters.next();
        }
    }
    output
}

struct Cursor<'a> {
    sql: &'a str,
    at: usize,
    location: Location,
}

impl Cursor<'_> {
    fn identifier_before(&self) -> bool {
        self.sql[..self.at]
            .chars()
            .next_back()
            .is_some_and(|previous| {
                previous.is_alphanumeric() || matches!(previous, '_' | '$') || !previous.is_ascii()
            })
    }
    fn peek(&self) -> Option<char> {
        self.sql[self.at..].chars().next()
    }
    fn starts(&self, prefix: &str) -> bool {
        self.sql[self.at..].starts_with(prefix)
    }
    fn next(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.at += character.len_utf8();
        if character == '\n' {
            self.location.line += 1;
            self.location.column = 1;
        } else {
            self.location.column += 1;
        }
        Some(character)
    }
    fn line_comment(&mut self) -> bool {
        while let Some(character) = self.next() {
            if character == '\n' {
                return true;
            }
        }
        false
    }
    fn block_comment(&mut self) {
        self.next();
        self.next();
        let mut depth = 1;
        while self.peek().is_some() {
            if self.starts("/*") {
                self.next();
                self.next();
                depth += 1;
            } else if self.starts("*/") {
                self.next();
                self.next();
                depth -= 1;
                if depth == 0 {
                    break;
                }
            } else {
                self.next();
            }
        }
    }
    fn dollar_body(&mut self) -> bool {
        let remaining = &self.sql[self.at + 1..];
        let Some(end) = remaining.find('$') else {
            return false;
        };
        let tag = &remaining[..end];
        if !tag.is_empty()
            && (!tag
                .chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_' || !c.is_ascii())
                || !tag
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || !c.is_ascii()))
        {
            return false;
        }
        let delimiter = &self.sql[self.at..self.at + end + 2];
        let after = self.at + delimiter.len();
        let finish = self.sql[after..]
            .find(delimiter)
            .map_or(self.sql.len(), |end| after + end + delimiter.len());
        while self.at < finish {
            self.next();
        }
        true
    }
    fn quoted(
        &mut self,
        quote: char,
        escaped: bool,
        mask: bool,
        output: &mut [u8],
    ) -> Option<usize> {
        self.next();
        while let Some(character) = self.peek() {
            let at = self.at;
            self.next();
            if escaped && character == '\\' {
                if mask && self.peek() == Some('\'') {
                    // A plain continuation tokenizer must not close on this quote.
                    output[self.at] = b'_';
                }
                self.next();
            } else if character == quote {
                if self.peek() == Some(quote) {
                    self.next();
                } else {
                    return Some(at);
                }
            }
        }
        None
    }
}
