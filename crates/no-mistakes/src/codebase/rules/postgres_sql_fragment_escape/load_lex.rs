use std::iter::Peekable;
use std::str::CharIndices;

mod slash;

use slash::{skip_slash, SlashKind};

pub(super) struct LexState {
    allows_regex: bool,
    ident_start: Option<usize>,
}

impl LexState {
    pub(super) fn new() -> Self {
        Self {
            allows_regex: true,
            ident_start: None,
        }
    }

    pub(super) fn reset_after_jump(&mut self) {
        self.allows_regex = false;
        self.ident_start = None;
    }

    pub(super) fn step(
        &mut self,
        text: &str,
        chars: &mut Peekable<CharIndices<'_>>,
        index: usize,
        ch: char,
    ) -> Option<bool> {
        if is_ident_continue(ch) {
            self.ident_start.get_or_insert(index);
            return Some(false);
        }
        self.finish_ident(text, index);
        if matches!(ch, '\'' | '"' | '`') {
            quoted_end(chars, ch, false)?;
            self.allows_regex = false;
            return Some(true);
        }
        if ch == '/' {
            if let Some(kind) = skip_slash(chars, self.allows_regex)? {
                if kind == SlashKind::Regex {
                    self.allows_regex = false;
                }
                return Some(true);
            }
            self.allows_regex = true;
            return Some(false);
        }
        if !ch.is_whitespace() {
            self.allows_regex = !matches!(ch, ')' | ']');
        }
        Some(false)
    }

    fn finish_ident(&mut self, text: &str, end: usize) {
        let Some(start) = self.ident_start.take() else {
            return;
        };
        self.allows_regex = matches!(
            &text[start..end],
            "return"
                | "case"
                | "throw"
                | "in"
                | "of"
                | "instanceof"
                | "typeof"
                | "void"
                | "delete"
                | "new"
                | "yield"
                | "await"
                | "else"
                | "do"
        );
    }
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}

fn quoted_end(chars: &mut Peekable<CharIndices<'_>>, quote: char, literal: bool) -> Option<usize> {
    while let Some((index, ch)) = chars.next() {
        if ch == '\u{5c}' {
            chars.next();
            continue;
        }
        if literal
            && quote == '`'
            && ch == '$'
            && chars.peek().is_some_and(|(_, next)| *next == '{')
        {
            return None;
        }
        if ch == quote {
            return Some(index + ch.len_utf8());
        }
    }
    None
}

pub(super) fn static_argument(text: &str) -> bool {
    let Some(text) = without_trivia(text) else {
        return false;
    };
    let mut chars = text.char_indices().peekable();
    let Some((_, quote @ ('\'' | '"' | '`'))) = chars.next() else {
        return false;
    };
    let Some(end) = quoted_end(&mut chars, quote, true) else {
        return false;
    };
    let Some(rest) = without_trivia(&text[end..]) else {
        return false;
    };
    rest.is_empty() || rest.starts_with([')', ',', '.'])
}

fn without_trivia(text: &str) -> Option<&str> {
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }
        if ch == '/' && skip_slash(&mut chars, false)?.is_some() {
            continue;
        }
        return Some(&text[index..]);
    }
    Some("")
}
