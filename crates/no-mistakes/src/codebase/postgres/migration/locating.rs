//! Match lexical heads to their AST anchor, never to another statement's name.
use sqlparser::{
    ast::Spanned,
    tokenizer::{Token, TokenWithSpan},
};

pub(super) struct Locations<'a> {
    tokens: &'a [TokenWithSpan],
}

impl<'a> Locations<'a> {
    pub(super) fn new(tokens: &'a [TokenWithSpan]) -> Self {
        Self { tokens }
    }

    pub(super) fn statement_line(&self, statement: &sqlparser::ast::Statement) -> usize {
        let anchor = match statement {
            sqlparser::ast::Statement::Drop { names, .. } => {
                names
                    .first()
                    .map(Spanned::span)
                    .unwrap_or_else(sqlparser::tokenizer::Span::empty)
                    .start
            }
            _ => statement.span().start,
        };
        let keyword = match statement {
            sqlparser::ast::Statement::Drop { .. } => "DROP",
            sqlparser::ast::Statement::AlterTable(_) => "ALTER",
            sqlparser::ast::Statement::Truncate(_) => "TRUNCATE",
            _ => "CREATE",
        };
        statement_keyword_line(self.tokens, keyword, anchor)
    }
}

pub(crate) fn statement_keyword_line(
    tokens: &[TokenWithSpan],
    keyword: &str,
    anchor: sqlparser::tokenizer::Location,
) -> usize {
    let before = tokens.partition_point(|token| token.span.start <= anchor);
    let start = tokens[..before]
        .iter()
        .rposition(|token| token.token == Token::SemiColon)
        .map_or(0, |at| at + 1);
    tokens.get(start..before).and_then(|tokens| tokens.iter().rev().find(|token| {
            matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(keyword))
        }))
            .map_or(anchor.line.max(1) as usize, |token| token.span.start.line as usize)
}
