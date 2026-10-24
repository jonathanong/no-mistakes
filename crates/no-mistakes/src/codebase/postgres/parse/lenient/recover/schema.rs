//! Locate the existing guarded schema/DML suffix without changing recovery semantics.
use super::super::{keyword_of, next_non_ws};
use super::partition::recover_partition_change;
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::TokenWithSpan;
use sqlparser::{keywords::Keyword, tokenizer::Token};

pub(super) fn recover(
    tokens: &[Token],
    original: Option<&[TokenWithSpan]>,
    allow_concurrent_detach: bool,
) -> Option<Statement> {
    let at = start(tokens)?;
    // PL/pgSQL wrappers can precede a complete partition transition.
    if let Some(change) = recover_partition_change(
        &tokens[at..],
        original.map(|tokens| &tokens[at..]),
        allow_concurrent_detach,
    ) {
        return Some(change);
    }
    let dialect = PostgreSqlDialect {};
    let mut parser = match original {
        Some(tokens) => Parser::new(&dialect).with_tokens_with_locations(tokens[at..].to_vec()),
        None => Parser::new(&dialect).with_tokens(tokens[at..].to_vec()),
    };
    parser.parse_statement().ok()
}

pub(super) fn start(tokens: &[Token]) -> Option<usize> {
    let mut index = 0;
    while index < tokens.len() {
        if let Some(start) = ddl_start_at(tokens, index) {
            return Some(start);
        }
        index += 1;
    }
    None
}

fn ddl_start_at(tokens: &[Token], index: usize) -> Option<usize> {
    match keyword_of(&tokens[index]) {
        Some(Keyword::ALTER) => (follows_keyword(tokens, index, Keyword::TABLE)
            || utility_start(tokens, index))
        .then_some(index),
        Some(Keyword::COMMENT) => utility_start(tokens, index).then_some(index),
        Some(Keyword::CREATE) => create_ddl_start(tokens, index),
        Some(Keyword::DROP) => drop_ddl_start(tokens, index),
        Some(
            Keyword::TRUNCATE
            | Keyword::INSERT
            | Keyword::UPDATE
            | Keyword::DELETE
            | Keyword::MERGE,
        ) => Some(index),
        _ => None,
    }
}

fn create_ddl_start(tokens: &[Token], index: usize) -> Option<usize> {
    let next = next_non_ws(tokens, index + 1)?;
    match keyword_of(&tokens[next]) {
        Some(Keyword::TABLE) | Some(Keyword::INDEX) | Some(Keyword::VIEW) => Some(index),
        Some(Keyword::UNIQUE) => follows_keyword(tokens, next, Keyword::INDEX).then_some(index),
        Some(Keyword::MATERIALIZED) => {
            follows_keyword(tokens, next, Keyword::VIEW).then_some(index)
        }
        Some(Keyword::OR | Keyword::TEMP | Keyword::TEMPORARY) => {
            (modified_view_start(tokens, next) || utility_start(tokens, index)).then_some(index)
        }
        _ => utility_start(tokens, index).then_some(index),
    }
}

fn modified_view_start(tokens: &[Token], mut next: usize) -> bool {
    if keyword_of(&tokens[next]) == Some(Keyword::OR) {
        let Some(replace) = next_non_ws(tokens, next + 1) else {
            return false;
        };
        if keyword_of(&tokens[replace]) != Some(Keyword::REPLACE) {
            return false;
        }
        let Some(after) = next_non_ws(tokens, replace + 1) else {
            return false;
        };
        next = after;
    }
    if matches!(
        keyword_of(&tokens[next]),
        Some(Keyword::TEMP | Keyword::TEMPORARY)
    ) {
        return follows_keyword(tokens, next, Keyword::VIEW);
    }
    keyword_of(&tokens[next]) == Some(Keyword::VIEW)
}

fn drop_ddl_start(tokens: &[Token], index: usize) -> Option<usize> {
    let next = next_non_ws(tokens, index + 1)?;
    match keyword_of(&tokens[next]) {
        Some(Keyword::INDEX) | Some(Keyword::TABLE) | Some(Keyword::VIEW) => Some(index),
        Some(Keyword::MATERIALIZED) => {
            follows_keyword(tokens, next, Keyword::VIEW).then_some(index)
        }
        _ => None,
    }
}

fn follows_keyword(tokens: &[Token], after: usize, expected: Keyword) -> bool {
    next_non_ws(tokens, after + 1)
        .and_then(|index| keyword_of(&tokens[index]))
        .is_some_and(|keyword| keyword == expected)
}

fn utility_start(tokens: &[Token], index: usize) -> bool {
    let code: Vec<_> = tokens[index..]
        .iter()
        .filter(|token| !matches!(token, Token::Whitespace(_)))
        .collect();
    let word = |at: usize, value: &str| matches!(code.get(at), Some(Token::Word(word)) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(value));
    let recognized = word(0, "COMMENT") && word(1, "ON")
        || word(0, "ALTER") && (word(1, "INDEX") || word(1, "TEXT") && word(2, "SEARCH"))
        || word(0, "CREATE")
            && (word(1, "INDEX")
                || word(1, "UNIQUE") && word(2, "INDEX")
                || word(1, "TYPE")
                || word(1, "TRIGGER")
                || word(1, "TEXT") && word(2, "SEARCH")
                || word(1, "OR") && word(2, "REPLACE") && word(3, "TRIGGER"));
    recognized && guarded_prefix(&tokens[..index])
}

fn guarded_prefix(prefix: &[Token]) -> bool {
    let first = prefix
        .iter()
        .find(|token| !matches!(token, Token::Whitespace(_)));
    if first.is_none() {
        return true;
    }
    if !matches!(first, Some(Token::Word(word)) if word.quote_style.is_none()
        && ["BEGIN", "ELSE", "IF", "ELSIF"].iter().any(|value| word.value.eq_ignore_ascii_case(value)))
    {
        return false;
    }
    let located: Vec<_> = prefix
        .iter()
        .cloned()
        .map(sqlparser::tokenizer::TokenWithSpan::wrap)
        .collect();
    super::procedural::BodyState::default().control_chunk(&located)
}

pub(super) fn recover(
    tokens: &[Token],
    original: Option<&[sqlparser::tokenizer::TokenWithSpan]>,
    allow_concurrent_detach: bool,
) -> Option<super::super::LocatedStatement> {
    use sqlparser::{dialect::PostgreSqlDialect, parser::Parser, tokenizer::TokenWithSpan};
    let start = start(tokens)?;
    if let Some(change) = super::partition::recover_partition_change(
        &tokens[start..],
        original.map(|tokens| &tokens[start..]),
        allow_concurrent_detach,
    ) {
        return Some(super::super::LocatedStatement::plain(change));
    }
    let mut located = original.map_or_else(
        || {
            tokens[start..]
                .iter()
                .cloned()
                .map(TokenWithSpan::wrap)
                .collect()
        },
        |tokens| tokens[start..].to_vec(),
    );
    super::super::super::ddl_compat::normalize(&mut located);
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(located);
    let statement = parser.parse_statement().ok()?;
    let utility = utility_start(tokens, start);
    if utility && parser.peek_token().token != Token::EOF {
        return None;
    }
    let mut located = super::super::LocatedStatement::plain(statement);
    // Only a validated wrapper plus a complete utility suffix establishes completeness.
    located.complete = utility;
    Some(located)
}
