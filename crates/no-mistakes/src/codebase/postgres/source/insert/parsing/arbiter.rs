use super::expressions::{self, Located};
use sqlparser::{
    ast::{Ident, ObjectName},
    parser::{Parser, ParserError},
    tokenizer::{Span, Token},
};

pub(in crate::codebase::postgres::source) struct Arbiter {
    pub expression: Located,
    pub operator_class: Option<(ObjectName, Span, Vec<(Ident, Located)>)>,
}

pub(super) fn parse(parser: &mut Parser<'_>) -> Result<Vec<Arbiter>, ParserError> {
    parser.next_token();
    let arbiters = parser.parse_comma_separated(|parser| {
        let expression = expressions::parse(parser)?;
        let operator_class = if matches!(parser.peek_token().token, Token::Word(_)) {
            let start = parser.peek_token().span.start;
            let name = parser.parse_object_name(false)?;
            let parameters = if parser.consume_token(&Token::LParen) {
                let parameters = parser.parse_comma_separated(|parser| {
                    let name = parser.parse_identifier()?;
                    parser.expect_token(&Token::Eq)?;
                    Ok((name, expressions::parse(parser)?))
                })?;
                parser.expect_token(&Token::RParen)?;
                parameters
            } else {
                Vec::new()
            };
            let end = parser.token_at(parser.index().saturating_sub(1)).span.end;
            Some((name, Span { start, end }, parameters))
        } else {
            None
        };
        Ok(Arbiter {
            expression,
            operator_class,
        })
    })?;
    parser.expect_token(&Token::RParen)?;
    Ok(arbiters)
}
