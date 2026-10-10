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
    let Some(lock) = tokens.first().and_then(|token| identifier_word(token)) else {
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
            .and_then(|token| identifier_word(token))
            .is_none()
        {
            return false;
        }
        *index += 1;
        while tokens
            .get(*index)
            .is_some_and(|token| matches!(token, Token::Period))
        {
            *index += 1;
            if tokens
                .get(*index)
                .and_then(|token| identifier_word(token))
                .is_none()
            {
                return false;
            }
            *index += 1;
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

fn identifier_word(token: &Token) -> Option<&str> {
    match token {
        Token::Word(word) => Some(word.value.as_str()),
        _ => None,
    }
}

fn keyword_word(token: &Token) -> Option<&str> {
    match token {
        Token::Word(word) if word.quote_style.is_none() => Some(word.value.as_str()),
        _ => None,
    }
}
