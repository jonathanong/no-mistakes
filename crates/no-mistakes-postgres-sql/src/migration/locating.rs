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

    pub(super) fn statement_line(
        &self,
        statement: &sqlparser::ast::Statement,
        fallback_start: Option<sqlparser::tokenizer::Location>,
    ) -> usize {
        let (anchor, strictly_before) = match statement {
            sqlparser::ast::Statement::Drop { names, .. } => (
                names
                    .first()
                    .map(Spanned::span)
                    .unwrap_or_else(sqlparser::tokenizer::Span::empty)
                    .start,
                false,
            ),
            sqlparser::ast::Statement::Savepoint { name }
            | sqlparser::ast::Statement::ReleaseSavepoint { name }
            | sqlparser::ast::Statement::Rollback {
                savepoint: Some(name),
                ..
            } => (name.span.start, true),
            _ => (statement.span().start, false),
        };
        let keyword = match statement {
            sqlparser::ast::Statement::Drop { .. } => "DROP",
            sqlparser::ast::Statement::AlterTable(_) => "ALTER",
            sqlparser::ast::Statement::Truncate(_) => "TRUNCATE",
            sqlparser::ast::Statement::StartTransaction { begin: true, .. } => "BEGIN",
            sqlparser::ast::Statement::StartTransaction { begin: false, .. } => "START",
            sqlparser::ast::Statement::Commit { end: false, .. } => "COMMIT",
            sqlparser::ast::Statement::Commit { end: true, .. } => "END",
            sqlparser::ast::Statement::Rollback { .. } => "ROLLBACK",
            sqlparser::ast::Statement::Savepoint { .. } => "SAVEPOINT",
            sqlparser::ast::Statement::ReleaseSavepoint { .. } => "RELEASE",
            _ => "CREATE",
        };
        if anchor == sqlparser::tokenizer::Location::empty() {
            if let Some(start) = fallback_start.filter(|loc| loc.line > 0) {
                return start.line as usize;
            }
            return self
                .tokens
                .iter()
                .find(|token| {
                    matches!(&token.token, Token::Word(word)
                        if word.quote_style.is_none()
                            && (word.value.eq_ignore_ascii_case(keyword)
                                || keyword == "ROLLBACK" && word.value.eq_ignore_ascii_case("ABORT")))
                })
                .map_or(1, |token| token.span.start.line.max(1) as usize);
        }
        keyword_position(self.tokens, keyword, anchor, strictly_before)
            .line
            .max(1) as usize
    }
}

pub(crate) fn statement_keyword_position(
    tokens: &[TokenWithSpan],
    keyword: &str,
    anchor: sqlparser::tokenizer::Location,
) -> sqlparser::tokenizer::Location {
    keyword_position(tokens, keyword, anchor, false)
}

fn keyword_position(
    tokens: &[TokenWithSpan],
    keyword: &str,
    anchor: sqlparser::tokenizer::Location,
    strictly_before: bool,
) -> sqlparser::tokenizer::Location {
    let before = tokens.partition_point(|token| {
        if strictly_before {
            token.span.start < anchor
        } else {
            token.span.start <= anchor
        }
    });
    let start = tokens[..before]
        .iter()
        .rposition(|token| token.token == Token::SemiColon)
        .map_or(0, |at| at + 1);
    tokens
        .get(start..before)
        .and_then(|tokens| {
            tokens.iter().rev().find(|token| {
                matches!(&token.token, Token::Word(word)
                    if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(keyword))
            })
        })
        .map_or(anchor, |token| token.span.start)
}
