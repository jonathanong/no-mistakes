use super::cursor::peek_index;
use super::literal::command_kind;
use super::scan::{adjust_paren, eq, word_of};
use super::Ctx;
use crate::codebase::postgres::source::types::PostgresSqlProceduralOccurrence;
use sqlparser::tokenizer::Token;

pub(super) fn occurrence(
    ctx: &mut Ctx<'_>,
    index: usize,
    paren: i32,
    cases: i32,
    stops: &[&str],
) -> Option<PostgresSqlProceduralOccurrence> {
    if paren != 0 || cases != 0 {
        return None;
    }
    // Only `FOR ... IN EXECUTE`. A qualified call such as `public.execute(...)`
    // is an identifier in the query, not this command slot.
    let execute = word_of(&ctx.tokens[index].token).is_some_and(|word| eq(word, "EXECUTE"));
    if !execute || !preceded_by_in(ctx, index) {
        return None;
    }
    Some(classified(ctx, index, stops))
}

fn preceded_by_in(ctx: &Ctx<'_>, index: usize) -> bool {
    let mut cursor = index;
    while cursor > 0 {
        cursor -= 1;
        let token = &ctx.tokens[cursor].token;
        if matches!(token, Token::Whitespace(_)) {
            continue;
        }
        return word_of(token).is_some_and(|word| eq(word, "IN"));
    }
    false
}

fn classified(ctx: &mut Ctx<'_>, start: usize, stops: &[&str]) -> PostgresSqlProceduralOccurrence {
    ctx.index = start + 1;
    let command = command_tokens(ctx, stops);
    let end = command.last().copied().unwrap_or(start);
    PostgresSqlProceduralOccurrence {
        kind: command_kind(ctx.tokens, &command),
        span: (ctx.span)(start, end),
        occurrences: Vec::new(),
    }
}

fn command_tokens(ctx: &mut Ctx<'_>, stops: &[&str]) -> Vec<usize> {
    let mut command = Vec::new();
    let mut paren = 0i32;
    while let Some(index) = peek_index(ctx) {
        let token = &ctx.tokens[index].token;
        if paren == 0 && ends_command(token, stops) {
            break;
        }
        command.push(index);
        adjust_paren(token, &mut paren);
        ctx.index = index + 1;
    }
    command
}

fn ends_command(token: &Token, stops: &[&str]) -> bool {
    if matches!(token, Token::SemiColon) {
        return true;
    }
    word_of(token).is_some_and(|word| {
        eq(word, "USING") || eq(word, "INTO") || stops.iter().any(|stop| eq(word, stop))
    })
}
