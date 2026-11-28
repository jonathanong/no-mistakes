mod commands;
mod nested;
mod redirection;
mod word;

use crate::codebase::command_lex::heredoc::Document;

pub(super) use commands::{Boundary, CaseArm};
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
    substitution_depth: usize,
    arith_unclosed: bool,
}

impl<'a> Lexer<'a> {
    pub(super) fn new(source: &'a str, shell: Shell) -> Self {
        Self {
            source,
            shell,
            offset: 0,
            line: 1,
            heredocs: Vec::new(),
            substitution_depth: 0,
            arith_unclosed: false,
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
