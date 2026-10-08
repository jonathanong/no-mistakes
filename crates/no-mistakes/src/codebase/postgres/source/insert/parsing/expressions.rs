//! Expression ASTs and delimiter positions share the prepared parser's tokens.
use sqlparser::{
    ast::Expr,
    parser::{Parser, ParserError},
    tokenizer::{Span, Token},
};

pub(in crate::codebase::postgres::source) struct Located {
    pub expression: Expr,
    pub span: Span,
    pub delimiters: Vec<Span>,
}

pub(super) fn parse(parser: &mut Parser<'_>) -> Result<Located, ParserError> {
    let index = parser.index();
    let start = parser.peek_token().span.start;
    let expression = parser.parse_expr()?;
    let end = parser.token_at(parser.index().saturating_sub(1)).span.end;
    let mut stack = Vec::new();
    let mut delimiters = Vec::new();
    for index in index..parser.index() {
        let token = parser.token_at(index);
        match token.token {
            Token::LParen | Token::LBracket => stack.push(token.span.start),
            Token::RParen | Token::RBracket => {
                delimiters.extend(stack.pop().map(|start| Span {
                    start,
                    end: token.span.end,
                }));
            }
            _ => {}
        }
    }
    Ok(Located {
        expression,
        span: Span { start, end },
        delimiters,
    })
}
