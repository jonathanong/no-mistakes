use super::super::keywords::{function_reserved, reserved, type_reserved};
use sqlparser::{parser::Parser, tokenizer::Token};
pub(super) struct Cursor<'a>(pub Parser<'a>);
impl Cursor<'_> {
    pub fn word(&mut self, expected: &str) -> bool {
        if matches!(self.0.peek_token().token, Token::Word(ref w)
            if w.quote_style.is_none() && w.value.eq_ignore_ascii_case(expected))
        {
            self.0.next_token();
            true
        } else {
            false
        }
    }
    pub fn words(&mut self, words: &[&str]) -> bool {
        if words.iter().enumerate().all(|(i, expected)| {
            matches!(self.0.peek_nth_token(i).token, Token::Word(ref w)
                if w.quote_style.is_none() && w.value.eq_ignore_ascii_case(expected))
        }) {
            for _ in words {
                self.0.next_token();
            }
            true
        } else {
            false
        }
    }

    pub fn identifier(&mut self) -> bool {
        if matches!(self.0.peek_token().token, Token::Word(ref w)
            if !w.value.is_empty() && (w.quote_style == Some('"') ||
                (w.quote_style.is_none() && !reserved(&w.value))))
        {
            self.0.next_token();
            true
        } else {
            false
        }
    }
    pub fn name(&mut self) -> bool {
        self.identifier() && (!self.token(Token::Period) || self.attribute())
    }
    fn attribute(&mut self) -> bool {
        if matches!(self.0.peek_token().token, Token::Word(ref w)
            if !w.value.is_empty() && (w.quote_style.is_none() || w.quote_style == Some('"')))
        {
            self.0.next_token();
            true
        } else {
            false
        }
    }
    pub fn function_name(&mut self) -> bool {
        // Qualified function names begin with ColId, then ColLabel attributes;
        // unqualified function names instead accept type_function_name.
        if self.0.peek_nth_token(1).token == Token::Period {
            return self.name();
        }
        if matches!(self.0.peek_token().token, Token::Word(ref w)
            if !w.value.is_empty() && (w.quote_style == Some('"') ||
                (w.quote_style.is_none() && !function_reserved(&w.value))))
        {
            self.0.next_token();
            true
        } else {
            false
        }
    }
    pub fn token(&mut self, token: Token) -> bool {
        self.0.consume_token(&token)
    }
    pub fn string(&mut self) -> bool {
        if matches!(
            self.0.peek_token().token,
            Token::SingleQuotedString(_)
                | Token::EscapedStringLiteral(_)
                | Token::UnicodeStringLiteral(_)
                | Token::DollarQuotedString(_)
        ) {
            self.0.next_token();
            true
        } else {
            false
        }
    }
    pub fn data_type(&mut self) -> bool {
        matches!(self.0.peek_token().token, Token::Word(ref w)
            if !w.value.is_empty() && (w.quote_style == Some('"') ||
                (w.quote_style.is_none() && !type_reserved(&w.value))))
            && self
                .0
                .parse_data_type()
                .is_ok_and(|ty| super::data_type::valid(&ty))
    }
    pub fn signature(&mut self) -> bool {
        if !self.function_name() {
            return false;
        }
        if !self.token(Token::LParen) {
            return true;
        }
        self.token(Token::RParen) || (self.types() && self.token(Token::RParen))
    }
    pub fn aggregate_signature(&mut self) -> bool {
        self.function_name()
            && self.token(Token::LParen)
            && (self.token(Token::Mul) || self.types())
            && self.token(Token::RParen)
    }
    pub fn types(&mut self) -> bool {
        if !self.data_type() {
            return false;
        }
        while self.token(Token::Comma) {
            if !self.data_type() {
                return false;
            }
        }
        true
    }
    pub fn drop_behavior(&mut self) {
        if !self.word("CASCADE") {
            self.word("RESTRICT");
        }
    }
}
