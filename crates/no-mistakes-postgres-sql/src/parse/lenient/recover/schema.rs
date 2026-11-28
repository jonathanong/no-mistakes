//! Locate the existing guarded schema/DML suffix without changing recovery semantics.
use super::super::{keyword_of, next_non_ws};
use sqlparser::{keywords::Keyword, tokenizer::Token};

pub(super) fn start_in(tokens: &[Token], procedural_body: bool, after_loop: bool) -> Option<usize> {
    (0..tokens.len()).find_map(|index| ddl_start_at(tokens, index, procedural_body, after_loop))
}

fn ddl_start_at(
    tokens: &[Token],
    index: usize,
    procedural_body: bool,
    after_loop: bool,
) -> Option<usize> {
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
        Some(Keyword::COMMIT | Keyword::ROLLBACK)
            if procedural_body
                && ((after_loop && index == 0)
                    || (index > 0 && guarded_prefix(&tokens[..index], true))) =>
        {
            Some(index)
        }
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
        let Some(replace) = next_non_ws(tokens, next + 1)
            .filter(|&i| keyword_of(&tokens[i]) == Some(Keyword::REPLACE))
        else {
            return false;
        };
        let Some(after) = next_non_ws(tokens, replace + 1) else {
            return false;
        };
        next = after;
    }
    match keyword_of(&tokens[next]) {
        Some(Keyword::TEMP | Keyword::TEMPORARY) => follows_keyword(tokens, next, Keyword::VIEW),
        Some(Keyword::VIEW) => true,
        _ => false,
    }
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
                || word(1, "SCHEMA")
                || word(1, "TRIGGER")
                || word(1, "TEXT") && word(2, "SEARCH")
                || word(1, "OR") && word(2, "REPLACE") && word(3, "TRIGGER"));
    recognized && guarded_prefix(&tokens[..index], false)
}

fn guarded_prefix(prefix: &[Token], allow_case_when: bool) -> bool {
    let Some(first) = prefix
        .iter()
        .find(|token| !matches!(token, Token::Whitespace(_)))
    else {
        return true;
    };
    let allowed: &[&str] = if allow_case_when {
        &["BEGIN", "ELSE", "IF", "ELSIF", "CASE", "WHEN"]
    } else {
        &["BEGIN", "ELSE", "IF", "ELSIF"]
    };
    if !matches!(first, Token::Word(word) if word.quote_style.is_none()
        && allowed.iter().any(|value| word.value.eq_ignore_ascii_case(value)))
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
    procedural_body: bool,
) -> Option<super::super::LocatedStatement> {
    use sqlparser::{dialect::PostgreSqlDialect, parser::Parser, tokenizer::TokenWithSpan};
    // A row-lock UPDATE belongs to the validated loop header, never its action.
    let action = procedural_body
        .then(|| {
            super::queries::loop_action_start(tokens)
                .unwrap_or_else(|()| super::queries::loop_header::fallback_action_start(tokens))
        })
        .flatten();
    let start = action.map_or_else(
        || start_in(tokens, procedural_body, false),
        |at| start_in(&tokens[at..], procedural_body, true).map(|start| at + start),
    )?;
    let is_txn_action = procedural_body
        && matches!(
            keyword_of(&tokens[start]),
            Some(Keyword::COMMIT | Keyword::ROLLBACK)
        );
    if action.is_some_and(|at| !guarded_prefix(&tokens[at..start], is_txn_action)) {
        return None;
    }
    if let Some((change, _)) = super::partition::recover_partition_change(
        &tokens[start..],
        original.map(|tokens| &tokens[start..]),
        allow_concurrent_detach,
    ) {
        return Some(super::super::LocatedStatement::plain(change));
    }
    let mut located = match original {
        Some(tokens) => tokens[start..].to_vec(),
        None => tokens[start..]
            .iter()
            .cloned()
            .map(TokenWithSpan::wrap)
            .collect(),
    };
    super::super::super::ddl_compat::normalize(&mut located);
    crate::source::conceal_body_keyword(&mut located);
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(located);
    let statement = parser.parse_statement().ok()?;
    if super::partition_sound::has_pg_partition(&statement)
        && (parser.peek_token().token != Token::EOF
            || !super::partition_sound::acceptable(&statement, allow_concurrent_detach))
    {
        return None;
    }
    let utility = utility_start(tokens, start) || is_txn_action;
    if (utility || action.is_some()) && parser.peek_token().token != Token::EOF {
        return None;
    }
    let mut located = super::super::LocatedStatement::plain(statement);
    // Only a validated wrapper plus a complete utility suffix establishes completeness.
    located.complete = utility;
    Some(located)
}
