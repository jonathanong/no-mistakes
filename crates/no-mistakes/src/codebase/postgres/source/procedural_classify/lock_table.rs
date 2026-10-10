use super::{cursor::peek_index, scan::eq, Ctx};
use sqlparser::tokenizer::Token;

/// Recognize the closed, static LOCK TABLE form needed by migration bootstrap code.
/// Other LOCK syntax remains unknown until its grammar has an explicit policy.
pub(super) fn is_static_lock_table(ctx: &Ctx<'_>) -> bool {
    let start = peek_index(ctx).unwrap_or(ctx.tokens.len());
    let tokens = ctx.tokens[start..]
        .iter()
        .map(|token| &token.token)
        .take_while(|token| !matches!(token, Token::SemiColon))
        .filter(|token| !matches!(token, Token::Whitespace(_)))
        .collect::<Vec<_>>();
    let Some(lock) = tokens.first().and_then(|token| keyword_word(token)) else {
        return false;
    };
    if !eq(lock, "LOCK")
        || !tokens
            .get(1)
            .and_then(|token| keyword_word(token))
            .is_some_and(|word| eq(word, "TABLE"))
    {
        return false;
    }
    let mut index = 2;
    if tokens
        .get(index)
        .and_then(|token| keyword_word(token))
        .is_some_and(|word| eq(word, "ONLY"))
    {
        index += 1;
    }
    if !consume_relations(&tokens, &mut index) {
        return false;
    }
    if !tokens
        .get(index)
        .and_then(|token| keyword_word(token))
        .is_some_and(|word| eq(word, "IN"))
    {
        return false;
    }
    index += 1;
    let mode_start = index;
    while tokens
        .get(index)
        .and_then(|token| keyword_word(token))
        .is_some_and(|word| !eq(word, "MODE"))
    {
        index += 1;
    }
    let mode = tokens[mode_start..index]
        .iter()
        .filter_map(|token| keyword_word(token))
        .map(str::to_ascii_uppercase)
        .collect::<Vec<_>>()
        .join(" ");
    if !matches!(
        mode.as_str(),
        "ACCESS SHARE"
            | "ROW SHARE"
            | "ROW EXCLUSIVE"
            | "SHARE UPDATE EXCLUSIVE"
            | "SHARE"
            | "SHARE ROW EXCLUSIVE"
            | "EXCLUSIVE"
            | "ACCESS EXCLUSIVE"
    ) || !tokens
        .get(index)
        .and_then(|token| keyword_word(token))
        .is_some_and(|word| eq(word, "MODE"))
    {
        return false;
    }
    index += 1;
    if tokens
        .get(index)
        .and_then(|token| keyword_word(token))
        .is_some_and(|word| eq(word, "NOWAIT"))
    {
        index += 1;
    }
    index == tokens.len()
}

fn consume_relations(tokens: &[&Token], index: &mut usize) -> bool {
    loop {
        if tokens
            .get(*index)
            .and_then(|token| relation_word(token))
            .is_none()
        {
            return false;
        }
        *index += 1;
        let mut components = 1;
        while tokens
            .get(*index)
            .is_some_and(|token| matches!(token, Token::Period))
        {
            if components == 3 {
                return false;
            }
            *index += 1;
            if tokens
                .get(*index)
                .and_then(|token| relation_word(token))
                .is_none()
            {
                return false;
            }
            *index += 1;
            components += 1;
        }
        if tokens
            .get(*index)
            .is_some_and(|token| matches!(token, Token::Comma))
        {
            *index += 1;
            continue;
        }
        return true;
    }
}

fn relation_word(token: &Token) -> Option<&str> {
    match token {
        Token::Word(word)
            if word.quote_style.is_some() || !postgres_non_col_id_keyword(&word.value) =>
        {
            Some(word.value.as_str())
        }
        _ => None,
    }
}

// PostgreSQL's RESERVED_KEYWORD and TYPE_FUNC_NAME_KEYWORD entries from
// src/include/parser/kwlist.h (REL_18_STABLE). Neither category can be a ColId,
// the identifier grammar used for each component of a relation name.
fn postgres_non_col_id_keyword(value: &str) -> bool {
    const NON_COL_ID: &str = "all analyse analyze and any array as asc asymmetric authorization binary both case cast check collate collation column concurrently constraint create cross current_catalog current_date current_role current_schema current_time current_timestamp current_user default deferrable desc distinct do else end except false fetch for foreign freeze from full grant group having ilike in initially inner intersect into is isnull join lateral leading left limit like localtime localtimestamp natural not notnull null offset on only or order outer overlaps placing primary references returning right select session_user similar some symmetric system_user table tablesample then to trailing true union unique user using variadic verbose when where window with";
    NON_COL_ID
        .split_ascii_whitespace()
        .any(|keyword| value.eq_ignore_ascii_case(keyword))
}

fn keyword_word(token: &Token) -> Option<&str> {
    match token {
        Token::Word(word) if word.quote_style.is_none() => Some(word.value.as_str()),
        _ => None,
    }
}
