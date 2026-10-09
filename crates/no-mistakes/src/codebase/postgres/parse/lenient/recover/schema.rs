//! Locate the existing guarded schema/DML suffix without changing recovery semantics.
use super::super::{keyword_of, next_non_ws};
use sqlparser::{keywords::Keyword, tokenizer::Token};

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
        Some(Keyword::ALTER) => follows_keyword(tokens, index, Keyword::TABLE).then_some(index),
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
            modified_view_start(tokens, next).then_some(index)
        }
        _ => None,
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
