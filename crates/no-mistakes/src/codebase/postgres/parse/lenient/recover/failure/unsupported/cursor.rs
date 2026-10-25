use super::super::keywords::reserved;
use sqlparser::tokenizer::Token;

pub(super) struct Cursor<'a> {
    pub(super) tokens: &'a [&'a Token],
    pub(super) at: usize,
}
impl Cursor<'_> {
    pub(super) fn word(&mut self, value: &str) -> bool {
        if matches!(self.tokens.get(self.at), Some(Token::Word(word))
            if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(value))
        {
            self.at += 1;
            true
        } else {
            false
        }
    }
    pub(super) fn words(&mut self, values: &[&str]) -> bool {
        let start = self.at;
        if values.iter().all(|value| self.word(value)) {
            true
        } else {
            self.at = start;
            false
        }
    }
    pub(super) fn identifier(&mut self) -> bool {
        // Reserved keywords must not masquerade as missing names or clauses.
        if matches!(self.tokens.get(self.at), Some(Token::Word(word))
            if !word.value.is_empty() && (word.quote_style == Some('"')
                || (word.quote_style.is_none() && !reserved(&word.value))))
        {
            self.at += 1;
            true
        } else {
            false
        }
    }
    pub(super) fn name(&mut self) -> bool {
        self.identifier() && (!self.period() || self.attribute())
    }
    fn attribute(&mut self) -> bool {
        if matches!(self.tokens.get(self.at), Some(Token::Word(word))
            if !word.value.is_empty() && (word.quote_style.is_none() || word.quote_style == Some('"')))
        {
            self.at += 1;
            true
        } else {
            false
        }
    }
    pub(super) fn identifiers(&mut self) -> bool {
        if !self.identifier() {
            return false;
        }
        while self.comma() {
            if !self.identifier() {
                return false;
            }
        }
        true
    }
    pub(super) fn names(&mut self) -> bool {
        if !self.name() {
            return false;
        }
        while self.comma() {
            if !self.name() {
                return false;
            }
        }
        true
    }
    pub(super) fn comma(&mut self) -> bool {
        if matches!(self.tokens.get(self.at), Some(Token::Comma)) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    pub(super) fn period(&mut self) -> bool {
        if matches!(self.tokens.get(self.at), Some(Token::Period)) {
            self.at += 1;
            true
        } else {
            false
        }
    }
}
