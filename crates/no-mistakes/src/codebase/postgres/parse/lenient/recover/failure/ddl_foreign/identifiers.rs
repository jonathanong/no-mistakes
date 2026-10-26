use super::super::super::keywords::{function_reserved, reserved};
use super::Cursor;
use sqlparser::tokenizer::Token;

impl Cursor<'_> {
    pub(in super::super) fn attribute(&mut self) -> bool {
        if matches!(self.tokens.get(self.at), Some(Token::Word(word))
            if !word.value.is_empty() && (word.quote_style.is_none() || word.quote_style == Some('"')))
        {
            self.at += 1;
            true
        } else {
            false
        }
    }
    pub(in super::super) fn function_name(&mut self) -> bool {
        if self
            .tokens
            .get(self.at + 1)
            .is_some_and(|token| **token == Token::Period)
        {
            return self.name();
        }
        if matches!(self.tokens.get(self.at), Some(Token::Word(word))
            if !word.value.is_empty() && (word.quote_style == Some('"')
                || word.quote_style.is_none() && !function_reserved(&word.value)))
        {
            self.at += 1;
            true
        } else {
            false
        }
    }
    pub(in super::super) fn identifier(&mut self) -> bool {
        let Some(Token::Word(word)) = self.tokens.get(self.at) else {
            return false;
        };
        if word.value.is_empty()
            || !(word.quote_style == Some('"')
                || (word.quote_style.is_none() && !reserved(&word.value)))
        {
            return false;
        }
        self.at += 1;
        true
    }
}
