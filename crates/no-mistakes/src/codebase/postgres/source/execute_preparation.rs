//! Normalize procedural EXECUTE grammar for conditional AST ownership.
use super::{locations::Locations, types::*};
use sqlparser::{
    dialect::PostgreSqlDialect,
    keywords::Keyword,
    parser::Parser,
    tokenizer::{Location, Token, TokenWithSpan, Word},
};
use std::collections::BTreeMap;

pub(super) type Occurrences = BTreeMap<Location, Result<PostgresSqlStatementKind, String>>;

pub(super) fn prepare(
    tokens: &mut Vec<TokenWithSpan>,
    source: &PostgresSqlSource,
    locations: &Locations<'_>,
    depth: usize,
    procedural: bool,
) -> Occurrences {
    let mut occurrences = Occurrences::new();
    if !procedural {
        return occurrences;
    }
    let mut original = std::mem::take(tokens).into_iter().peekable();
    let mut previous = None;
    let mut case_depth = 0usize;
    while let Some(token) = original.next() {
        if let Token::Word(word) = &token.token {
            if word.quote_style.is_none() && word.keyword == Keyword::CASE {
                case_depth += 1;
            }
            if word.quote_style.is_none() && word.keyword == Keyword::END {
                case_depth = case_depth.saturating_sub(1);
            }
        }
        let boundary = case_depth == 0 && previous.as_ref().is_none_or(|token: &Token| matches!(token, Token::SemiColon) || matches!(token, Token::Word(word) if word.quote_style.is_none() && [Keyword::THEN, Keyword::ELSE, Keyword::BEGIN].contains(&word.keyword)));
        let execute = matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && word.keyword == Keyword::EXECUTE);
        if boundary && execute {
            let mut owned = vec![token.clone()];
            while original
                .peek()
                .is_some_and(|next| next.token != Token::SemiColon)
            {
                owned.push(original.next().unwrap());
            }
            let end = owned
                .iter()
                .rev()
                .find(|value| !matches!(value.token, Token::Whitespace(_)))
                .unwrap()
                .span;
            let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(owned);
            occurrences.insert(
                token.span.start,
                super::execute::collect(&mut parser, source, locations, depth),
            );
            // The AST placeholder owns the same command interval; its fact is
            // replaced once from this map, so fabricated SELECT facts never escape.
            tokens.push(TokenWithSpan {
                token: Token::Word(Word {
                    value: "SELECT".into(),
                    quote_style: None,
                    keyword: Keyword::SELECT,
                }),
                span: token.span,
            });
            tokens.push(TokenWithSpan {
                token: Token::Number("1".into(), false),
                span: end,
            });
            previous = Some(Token::Number("1".into(), false));
        } else {
            if !matches!(token.token, Token::Whitespace(_)) {
                previous = Some(token.token.clone());
            }
            tokens.push(token);
        }
    }
    occurrences
}
