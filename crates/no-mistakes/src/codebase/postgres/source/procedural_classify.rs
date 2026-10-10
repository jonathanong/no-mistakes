//! Structural PL/pgSQL walk. Source occurrences are not executed statements.
mod controls;
mod cursor;
mod header_execute;
mod literal;
mod scan;
mod select_gate;
mod statements;

use super::types::*;
use sqlparser::tokenizer::TokenWithSpan;

pub(super) struct Classified {
    pub occurrences: Vec<PostgresSqlProceduralOccurrence>,
    pub legacy_stop: bool,
    pub walker_only: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Stop {
    Outer,
    IfBranch,
    Loop,
    CaseBranch,
    Handler,
    None,
}

pub(super) struct Ctx<'a> {
    pub(super) tokens: &'a [TokenWithSpan],
    pub(super) index: usize,
    pub(super) depth: u8,
    pub(super) legacy_stop: bool,
    pub(super) walker_only: bool,
    pub(super) span: &'a dyn Fn(usize, usize) -> PostgresSqlSpan,
}

pub(super) fn classify(
    tokens: &[TokenWithSpan],
    span: &dyn Fn(usize, usize) -> PostgresSqlSpan,
    rooted: bool,
) -> Classified {
    classify_at(tokens, span, rooted, 0)
}

pub(super) fn classify_at(
    tokens: &[TokenWithSpan],
    span: &dyn Fn(usize, usize) -> PostgresSqlSpan,
    rooted: bool,
    depth: u8,
) -> Classified {
    let mut ctx = Ctx {
        tokens,
        index: 0,
        depth,
        legacy_stop: false,
        walker_only: false,
        span,
    };
    cursor::skip_label(&mut ctx);
    let occurrences = if !rooted {
        walk_statements(&mut ctx, Stop::None)
    } else if !cursor::eat_word(&mut ctx, "BEGIN") {
        ctx.legacy_stop = true;
        walk_statements(&mut ctx, Stop::None)
    } else {
        walk_block(&mut ctx)
    };
    Classified {
        occurrences,
        legacy_stop: ctx.legacy_stop,
        walker_only: ctx.walker_only,
    }
}

fn walk_block(ctx: &mut Ctx<'_>) -> Vec<PostgresSqlProceduralOccurrence> {
    let mut occurrences = walk_statements(ctx, Stop::Outer);
    if cursor::at_word(ctx, "EXCEPTION") {
        let start = cursor::peek_index(ctx).unwrap();
        cursor::eat_word(ctx, "EXCEPTION");
        ctx.walker_only = true;
        occurrences.push(controls::exception(ctx, start));
    }
    if cursor::at_plain_end(ctx) {
        cursor::eat_word(ctx, "END");
        cursor::eat_label(ctx);
        cursor::eat_semi(ctx);
        if cursor::peek_index(ctx).is_some() {
            ctx.legacy_stop = true;
            occurrences.push(statements::rest_unknown(ctx));
        }
    } else {
        ctx.legacy_stop = true;
        occurrences.push(statements::rest_unknown(ctx));
    }
    occurrences
}

pub(super) fn walk_statements(
    ctx: &mut Ctx<'_>,
    stop: Stop,
) -> Vec<PostgresSqlProceduralOccurrence> {
    let mut occurrences = Vec::new();
    while cursor::peek_index(ctx).is_some() && !cursor::stopped(ctx, stop) {
        // Skip every leading semicolon. One leftover `;` would be an unknown
        // statement whose scan consumes the following statement.
        while cursor::eat_semi(ctx) {}
        cursor::skip_label(ctx);
        if cursor::peek_index(ctx).is_none() || cursor::stopped(ctx, stop) {
            break;
        }
        occurrences.push(statements::statement(ctx));
    }
    occurrences
}

pub(super) fn done(
    ctx: &Ctx<'_>,
    kind: PostgresSqlProceduralOccurrenceKind,
    start: usize,
    occurrences: Vec<PostgresSqlProceduralOccurrence>,
) -> PostgresSqlProceduralOccurrence {
    PostgresSqlProceduralOccurrence {
        kind,
        span: (ctx.span)(start, cursor::last_index(ctx, start)),
        occurrences,
    }
}
