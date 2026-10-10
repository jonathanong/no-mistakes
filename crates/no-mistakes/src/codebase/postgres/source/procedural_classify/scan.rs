use super::cursor::peek_index;
use super::Ctx;
use crate::codebase::postgres::source::types::{
    PostgresSqlProceduralOccurrence, PostgresSqlProceduralOccurrenceKind,
};
use sqlparser::tokenizer::Token;

type Kind = PostgresSqlProceduralOccurrenceKind;

pub(super) fn scan_header(
    ctx: &mut Ctx<'_>,
    stops: &[&str],
) -> Vec<PostgresSqlProceduralOccurrence> {
    let mut extra = Vec::new();
    let mut paren = 0i32;
    let mut cases = 0i32;
    while let Some(index) = peek_index(ctx) {
        // `take_command` would swallow a header stop such as LOOP.
        if let Some(occurrence) = super::header_execute::occurrence(ctx, index, paren, cases, stops)
        {
            extra.push(occurrence);
            continue;
        }
        let token = &ctx.tokens[index].token;
        if let Some(word) = word_of(token) {
            if paren == 0 && cases == 0 && stops.iter().any(|stop| eq(word, stop)) {
                break;
            }
            if paren == 0 && eq(word, "CASE") {
                cases += 1;
            } else if paren == 0 && cases > 0 && eq(word, "END") {
                cases -= 1;
            } else if is_dml(word) {
                extra.push(token_occurrence(ctx, Kind::Dml, index));
            } else if eq(word, "EXECUTE") {
                extra.push(token_occurrence(ctx, Kind::DynamicExecute, index));
            }
        } else {
            adjust_paren(token, &mut paren);
        }
        ctx.index = index + 1;
    }
    extra
}

pub(super) fn consume_statement(ctx: &mut Ctx<'_>, cte: bool) -> (bool, bool) {
    let mut paren = 0i32;
    let mut cases = 0i32;
    let mut saw_dml = false;
    let mut saw_dynamic = false;
    let mut arm_cte = false;
    while let Some(index) = peek_index(ctx) {
        let token = &ctx.tokens[index].token;
        if paren == 0 && cases == 0 && matches!(token, Token::SemiColon) {
            ctx.index = index + 1;
            break;
        }
        if let Some(word) = word_of(token) {
            if paren == 0 && eq(word, "CASE") {
                cases += 1;
            } else if paren == 0 && cases > 0 && eq(word, "END") {
                cases -= 1;
            }
            if cte && arm_cte && is_dml(word) {
                saw_dml = true;
            }
            if cte && paren == 0 && cases == 0 && is_dml(word) {
                saw_dml = true;
            }
            if cte && paren == 0 && eq(word, "EXECUTE") {
                saw_dynamic = true;
            }
            arm_cte = cte && eq(word, "AS");
        } else if matches!(token, Token::LParen) && arm_cte {
            paren += 1;
        } else {
            arm_cte = false;
            adjust_paren(token, &mut paren);
        }
        ctx.index = index + 1;
    }
    (saw_dml, saw_dynamic)
}

pub(super) fn collect_keywords_until_semi(
    ctx: &mut Ctx<'_>,
) -> Vec<PostgresSqlProceduralOccurrence> {
    let mut nested = Vec::new();
    let mut paren = 0i32;
    while let Some(index) = peek_index(ctx) {
        let token = &ctx.tokens[index].token;
        if paren == 0 && matches!(token, Token::SemiColon) {
            ctx.index = index + 1;
            break;
        }
        if let Some(word) = word_of(token) {
            if is_dml(word) {
                nested.push(token_occurrence(ctx, Kind::Dml, index));
            } else if eq(word, "EXECUTE") {
                nested.push(token_occurrence(ctx, Kind::DynamicExecute, index));
            }
        } else {
            adjust_paren(token, &mut paren);
        }
        ctx.index = index + 1;
    }
    nested
}

pub(super) fn collect_keywords_rest(ctx: &mut Ctx<'_>) -> Vec<PostgresSqlProceduralOccurrence> {
    let mut nested = Vec::new();
    while let Some(index) = peek_index(ctx) {
        if let Some(word) = word_of(&ctx.tokens[index].token) {
            if is_dml(word) {
                nested.push(token_occurrence(ctx, Kind::Dml, index));
            } else if eq(word, "EXECUTE") {
                nested.push(token_occurrence(ctx, Kind::DynamicExecute, index));
            }
        }
        ctx.index = index + 1;
    }
    nested
}

pub(super) fn take_command(ctx: &mut Ctx<'_>) -> Vec<usize> {
    let mut command = Vec::new();
    let mut paren = 0i32;
    while let Some(index) = peek_index(ctx) {
        let token = &ctx.tokens[index].token;
        if paren == 0
            && (matches!(token, Token::SemiColon)
                || word_of(token).is_some_and(|word| eq(word, "USING") || eq(word, "INTO")))
        {
            break;
        }
        command.push(index);
        adjust_paren(token, &mut paren);
        ctx.index = index + 1;
    }
    command
}

pub(super) fn token_occurrence(
    ctx: &Ctx<'_>,
    kind: Kind,
    index: usize,
) -> PostgresSqlProceduralOccurrence {
    PostgresSqlProceduralOccurrence {
        kind,
        span: (ctx.span)(index, index),
        occurrences: Vec::new(),
    }
}

pub(super) fn word_of(token: &Token) -> Option<&str> {
    match token {
        Token::Word(word) if word.quote_style.is_none() => Some(word.value.as_str()),
        _ => None,
    }
}

pub(super) fn eq(word: &str, expected: &str) -> bool {
    word.eq_ignore_ascii_case(expected)
}

pub(super) fn is_dml(word: &str) -> bool {
    matches!(
        word.to_ascii_uppercase().as_str(),
        "INSERT" | "UPDATE" | "DELETE" | "MERGE"
    )
}

pub(super) fn is_utility(word: &str) -> bool {
    matches!(
        word.to_ascii_uppercase().as_str(),
        "CREATE"
            | "ALTER"
            | "DROP"
            | "COMMENT"
            | "GRANT"
            | "REVOKE"
            | "SET"
            | "RESET"
            | "ANALYZE"
            | "VACUUM"
            | "REINDEX"
            | "CLUSTER"
            | "DISCARD"
            | "REFRESH"
    )
}

/// Recognize the closed, static LOCK TABLE form needed by migration bootstrap code.
/// Other LOCK syntax remains unknown until its grammar has an explicit policy.
pub(super) fn is_static_lock_table(ctx: &Ctx<'_>) -> bool {
    let start = peek_index(ctx).unwrap_or(ctx.tokens.len());
    lock_table_tail(ctx, start)
}

fn lock_table_tail(ctx: &Ctx<'_>, start: usize) -> bool {
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
    if !consume_lock_relations(&tokens, &mut index) {
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

fn consume_lock_relations(tokens: &[&Token], index: &mut usize) -> bool {
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

pub(super) fn adjust_paren(token: &Token, paren: &mut i32) {
    match token {
        Token::LParen | Token::LBracket => *paren += 1,
        Token::RParen | Token::RBracket => *paren = paren.saturating_sub(1),
        _ => {}
    }
}
