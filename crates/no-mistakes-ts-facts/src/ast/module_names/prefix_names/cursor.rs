//! Tokens for the deliberately narrow module-prefix grammar, not a JS lexer.
use oxc_span::Span;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Token<'a> {
    Word(&'a str),
    Quoted(Span, bool),
    Punctuation(char),
    End,
}

pub(super) struct Cursor<'a> {
    source: &'a str,
    position: usize,
    limit: usize,
    pending: Option<Token<'a>>,
}

impl<'a> Cursor<'a> {
    pub(super) fn new(source: &'a str, limit: usize) -> Self {
        Self {
            source,
            position: 0,
            limit,
            pending: None,
        }
    }

    // Statement boundaries have consumed all lookahead. Quoted expressions are
    // unsupported here, so decline before inspecting their contents.
    pub(super) fn statement(&mut self) -> Option<Token<'a>> {
        self.trivia()?;
        if matches!(self.character(), Some('\'' | '"')) {
            return None;
        }
        self.next()
    }

    pub(super) fn peek(&mut self) -> Option<Token<'a>> {
        if self.pending.is_none() {
            self.pending = self.read();
        }
        self.pending
    }

    pub(super) fn next(&mut self) -> Option<Token<'a>> {
        self.pending.take().or_else(|| self.read())
    }

    fn read(&mut self) -> Option<Token<'a>> {
        self.trivia()?;
        let Some(character) = self.character() else {
            return Some(Token::End);
        };
        let start = self.position;
        if character.is_ascii_alphabetic() || matches!(character, '_' | '$') {
            self.position += 1;
            while self
                .character()
                .is_some_and(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '$'))
            {
                self.position += 1;
            }
            return Some(Token::Word(&self.source[start..self.position]));
        }
        if matches!(character, '\'' | '"') {
            return self.quoted(character);
        }
        if matches!(character, '{' | '}' | ',' | ';' | '.' | '*') {
            self.position += 1;
            return Some(Token::Punctuation(character));
        }
        None
    }

    fn character(&self) -> Option<char> {
        self.source.get(self.position..self.limit)?.chars().next()
    }

    fn trivia(&mut self) -> Option<()> {
        loop {
            while let Some(character) = self
                .character()
                .filter(|character| super::super::is_trivia_space(*character))
            {
                self.position += character.len_utf8();
            }
            let remaining = self.source.get(self.position..self.limit)?;
            if remaining.starts_with("//") {
                self.position += remaining
                    .find(['\n', '\r', '\u{2028}', '\u{2029}'])
                    .unwrap_or(remaining.len());
            } else if remaining.starts_with("/*") {
                self.position += remaining.find("*/")? + 2;
            } else {
                return Some(());
            }
        }
    }

    fn quoted(&mut self, quote: char) -> Option<Token<'a>> {
        let start = self.position;
        self.position += 1;
        loop {
            let character = self.character()?;
            self.position += character.len_utf8();
            match character {
                c if c == quote => {
                    let raw = &self.source[start + 1..self.position - 1];
                    let lone = super::super::super::js_string::raw_has_lone_surrogate(raw)?;
                    return Some(Token::Quoted(
                        Span::new(start as u32, self.position as u32),
                        lone,
                    ));
                }
                '\r' | '\n' => return None,
                '\\' => {
                    let escaped = self.character()?;
                    self.position += escaped.len_utf8();
                    // Octal and identity escapes are outside this proven prefix.
                    if !matches!(
                        escaped,
                        '\\' | '\''
                            | '"'
                            | 'u'
                            | 'x'
                            | 'n'
                            | 'r'
                            | 't'
                            | 'b'
                            | 'f'
                            | 'v'
                            | '\n'
                            | '\r'
                            | '\u{2028}'
                            | '\u{2029}'
                    ) {
                        return None;
                    }
                    if escaped == '\r' && self.character() == Some('\n') {
                        self.position += 1;
                    }
                }
                _ => {}
            }
        }
    }
}
