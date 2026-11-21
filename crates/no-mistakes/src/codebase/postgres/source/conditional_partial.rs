//! Procedural conditionals skip the top-level conflict-target preparation.
//! A `WHERE` between `ON CONFLICT` and `DO` is a partial-index predicate.
//! Projecting the insert without that predicate would name the wrong arbiter.
use sqlparser::{
    keywords::Keyword,
    tokenizer::{Token, TokenWithSpan},
};

pub(super) fn partial_conflict(tokens: &[&TokenWithSpan]) -> bool {
    let significant = tokens
        .iter()
        .copied()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .collect::<Vec<_>>();
    let mut conflict = false;
    let mut index = 0;
    while index < significant.len() {
        if index + 1 < significant.len()
            && keyword(significant[index], Keyword::ON)
            && keyword(significant[index + 1], Keyword::CONFLICT)
        {
            conflict = true;
            index += 2;
            continue;
        }
        if conflict && keyword(significant[index], Keyword::DO) {
            return false;
        }
        if conflict && keyword(significant[index], Keyword::WHERE) {
            return true;
        }
        index += 1;
    }
    false
}

fn keyword(token: &TokenWithSpan, expected: Keyword) -> bool {
    matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && word.keyword == expected)
}
