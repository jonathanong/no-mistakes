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

pub(super) fn adjust_paren(token: &Token, paren: &mut i32) {
    match token {
        Token::LParen | Token::LBracket => *paren += 1,
        Token::RParen | Token::RBracket => *paren = paren.saturating_sub(1),
        _ => {}
    }
}
