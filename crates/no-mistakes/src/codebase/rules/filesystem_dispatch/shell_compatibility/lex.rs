mod nested;
mod redirection;
mod word;

use crate::codebase::command_lex::heredoc::Document;

pub(super) use word::{assignment, identifier, Word};

pub(super) enum Shell {
    Bash,
    Zsh,
}

pub(super) struct Lexer<'a> {
    source: &'a str,
    shell: Shell,
    offset: usize,
    pub(super) line: usize,
    heredocs: Vec<Document>,
}

impl<'a> Lexer<'a> {
    pub(super) fn new(source: &'a str, shell: Shell) -> Self {
        Self {
            source,
            shell,
            offset: 0,
            line: 1,
            heredocs: Vec::new(),
        }
    }

    fn current(&self) -> Option<u8> {
        self.source.as_bytes().get(self.offset).copied()
    }

    fn rest(&self) -> &'a str {
        &self.source[self.offset..]
    }

    fn advance(&mut self) {
        let ch = self
            .rest()
            .chars()
            .next()
            .expect("lexer has input to advance");
        self.line += usize::from(ch == '\n');
        self.offset += ch.len_utf8();
    }

    fn take(&mut self, token: &str) -> bool {
        if self.rest().starts_with(token) {
            self.offset += token.len();
            true
        } else {
            false
        }
    }

    fn continuation(&mut self) -> bool {
        if self.take("\\\n") || self.take("\\\r\n") {
            self.line += 1;
            true
        } else {
            false
        }
    }

    fn skip_comment(&mut self) {
        while self.current().is_some_and(|byte| byte != b'\n') {
            self.advance();
        }
    }

    pub(super) fn commands(mut self, mut visit: impl FnMut(&[Word<'a>], usize)) {
        let mut words = Vec::new();
        let mut scope = 0usize;
        let mut groups = Vec::new();
        let mut pending_function = false;
        let mut conditional = false;
        while let Some(byte) = self.current() {
            if self.continuation() {
                continue;
            }
            match byte {
                b' ' | b'\t' | b'\r' | b'\x0c' => self.advance(),
                b'#' => self.skip_comment(),
                b'\n' | b';' | b'|' | b'&' | b'(' | b')' | b'<' | b'>' if conditional => {
                    self.advance();
                }
                b'{' | b'}' if self.group_boundary(&words) => {
                    let function_body = pending_function || starts_with_function(&words);
                    pending_function = false;
                    visit(&words, scope);
                    words.clear();
                    self.advance();
                    if byte == b'{' {
                        groups.push(function_body);
                        scope += usize::from(function_body);
                    } else {
                        scope -= usize::from(groups.pop().unwrap_or_default());
                        visit(&[], scope);
                    }
                }
                b'&' if self.rest().starts_with("&>") => words.extend(self.redirection()),
                b'(' if function_parentheses(&words) && self.take("()") => {
                    pending_function = true;
                    visit(&words, scope);
                    words.clear();
                }
                b'\n' | b';' | b'|' | b'&' | b'(' | b')' => {
                    pending_function = matches!(byte, b'\n' | b';')
                        && (pending_function || starts_with_function(&words));
                    visit(&words, scope);
                    words.clear();
                    if self.take("((") {
                        self.skip_group(b')');
                        self.take(")");
                        continue;
                    }
                    self.advance();
                    if byte == b'\n' {
                        self.skip_heredocs();
                    }
                }
                b'<' | b'>' => {
                    if let Some(word) = self.redirection() {
                        words.push(word);
                    }
                }
                _ => {
                    pending_function = false;
                    let mut word = self.word();
                    word.redirect = !conditional && self.is_fd_prefix(word.raw);
                    if word.raw == "[[" && command_words(&words).next().is_none() {
                        conditional = true;
                    } else if word.raw == "]]" {
                        conditional = false;
                    }
                    words.push(word);
                }
            }
        }
        visit(&words, scope);
    }

    fn group_boundary(&self, words: &[Word<'_>]) -> bool {
        if command_words(words)
            .next()
            .is_some_and(|word| word.raw != "function")
        {
            return false;
        }
        self.source
            .as_bytes()
            .get(self.offset + 1)
            .is_none_or(|byte| {
                byte.is_ascii_whitespace() || matches!(byte, b';' | b'&' | b'|' | b')')
            })
    }
}

fn starts_with_function(words: &[Word<'_>]) -> bool {
    command_words(words)
        .next()
        .is_some_and(|word| word.raw == "function")
}

fn function_parentheses(words: &[Word<'_>]) -> bool {
    let mut words = command_words(words);
    words
        .next()
        .is_none_or(|word| word.raw == "function" || words.next().is_none())
}

pub(super) fn command_words<'w, 'a>(words: &'w [Word<'a>]) -> impl Iterator<Item = &'w Word<'a>> {
    words
        .iter()
        .filter(|word| !word.redirect)
        .skip_while(|word| {
            matches!(
                word.raw,
                "!" | "if" | "then" | "elif" | "else" | "while" | "until" | "do"
            )
        })
}
