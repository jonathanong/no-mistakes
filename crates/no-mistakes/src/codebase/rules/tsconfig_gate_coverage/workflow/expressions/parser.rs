use super::{lexer::Token, StaticExpressionType};

pub(super) fn parse(tokens: &[Token]) -> Option<StaticExpressionType> {
    Parser::new(tokens).parse()
}

struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, index: 0 }
    }

    fn parse(mut self) -> Option<StaticExpressionType> {
        let expression_type = self.parse_or()?;
        (self.index == self.tokens.len()).then_some(expression_type)
    }

    fn parse_or(&mut self) -> Option<StaticExpressionType> {
        let mut expression_type = self.parse_and()?;
        while self.take(Token::Or) {
            self.parse_and()?;
            expression_type = StaticExpressionType::Dynamic;
        }
        Some(expression_type)
    }

    fn parse_and(&mut self) -> Option<StaticExpressionType> {
        let mut expression_type = self.parse_comparison()?;
        while self.take(Token::And) {
            self.parse_comparison()?;
            expression_type = StaticExpressionType::Dynamic;
        }
        Some(expression_type)
    }

    fn parse_comparison(&mut self) -> Option<StaticExpressionType> {
        let mut expression_type = self.parse_unary()?;
        while self.take(Token::Comparison) {
            self.parse_unary()?;
            expression_type = StaticExpressionType::Boolean;
        }
        Some(expression_type)
    }

    fn parse_unary(&mut self) -> Option<StaticExpressionType> {
        if self.take(Token::Bang) {
            self.parse_unary()?;
            Some(StaticExpressionType::Boolean)
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Option<StaticExpressionType> {
        match self.next()? {
            Token::Boolean => Some(StaticExpressionType::Boolean),
            Token::Number => Some(StaticExpressionType::Number),
            Token::String => Some(StaticExpressionType::String),
            Token::Null => Some(StaticExpressionType::Null),
            Token::Identifier => {
                if self.take(Token::LeftParen) {
                    self.parse_arguments()?;
                }
                self.parse_accessors()
            }
            Token::LeftParen => {
                let expression_type = self.parse_or()?;
                self.take(Token::RightParen).then_some(expression_type)
            }
            _ => None,
        }
    }

    fn parse_accessors(&mut self) -> Option<StaticExpressionType> {
        loop {
            if self.take(Token::Dot) {
                if !self.take(Token::Identifier) && !self.take(Token::Star) {
                    return None;
                }
            } else if self.take(Token::LeftBracket) {
                self.parse_or()?;
                if !self.take(Token::RightBracket) {
                    return None;
                }
            } else {
                return Some(StaticExpressionType::Dynamic);
            }
        }
    }

    fn parse_arguments(&mut self) -> Option<()> {
        if self.take(Token::RightParen) {
            return Some(());
        }
        loop {
            self.parse_or()?;
            if self.take(Token::RightParen) {
                return Some(());
            }
            if !self.take(Token::Comma) {
                return None;
            }
        }
    }

    fn take(&mut self, token: Token) -> bool {
        if self.tokens.get(self.index) == Some(&token) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn next(&mut self) -> Option<Token> {
        let token = *self.tokens.get(self.index)?;
        self.index += 1;
        Some(token)
    }
}
