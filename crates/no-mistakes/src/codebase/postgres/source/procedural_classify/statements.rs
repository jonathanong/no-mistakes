use super::cursor::{at_any, at_dml, at_utility, at_word, bump, peek_index};
use super::scan::{collect_keywords_rest, consume_statement, is_static_lock_table};
use super::{controls, done, Ctx};
use crate::codebase::postgres::source::types::{
    PostgresSqlProceduralOccurrence, PostgresSqlProceduralOccurrenceKind as Kind,
};

pub(super) fn statement(ctx: &mut Ctx<'_>) -> PostgresSqlProceduralOccurrence {
    if ctx.depth >= 64 {
        return rest_unknown(ctx);
    }
    let label = std::mem::replace(&mut ctx.pending_label, super::cursor::OpenLabel::Absent);
    ctx.depth += 1;
    let occurrence = if at_any(ctx, &["LOOP", "WHILE", "FOR", "FOREACH"]) {
        controls::loop_stmt(ctx, label)
    } else if at_word(ctx, "BEGIN") {
        controls::begin_stmt(ctx, label)
    } else {
        // A label before RAISE or other non-block statements has no closer.
        super::cursor::record_invalid_label(ctx, &label);
        plain_statement(ctx)
    };
    ctx.depth -= 1;
    occurrence
}

fn plain_statement(ctx: &mut Ctx<'_>) -> PostgresSqlProceduralOccurrence {
    if at_word(ctx, "IF") {
        controls::if_stmt(ctx)
    } else if at_word(ctx, "CASE") {
        controls::case_stmt(ctx)
    } else if at_word(ctx, "RAISE") {
        controls::raise_stmt(ctx)
    } else if at_word(ctx, "EXECUTE") {
        controls::execute_stmt(ctx)
    } else if at_word(ctx, "DECLARE") {
        ctx.legacy_stop = true;
        simple(ctx, Kind::Unknown)
    } else if at_word(ctx, "DO") {
        simple(ctx, Kind::ControlFlow)
    } else if at_dml(ctx) {
        simple(ctx, Kind::Dml)
    } else if is_static_lock_table(ctx) {
        simple(ctx, Kind::Utility)
    } else if at_word(ctx, "WITH") {
        with_stmt(ctx)
    } else if at_utility(ctx) {
        simple(ctx, Kind::Utility)
    } else {
        simple(ctx, Kind::Unknown)
    }
}

fn with_stmt(ctx: &mut Ctx<'_>) -> PostgresSqlProceduralOccurrence {
    let start = peek_index(ctx).unwrap_or(ctx.index);
    bump(ctx);
    let (dml, dynamic) = consume_statement(ctx, true);
    let kind = if dynamic {
        Kind::DynamicExecute
    } else if dml {
        Kind::Dml
    } else {
        Kind::Unknown
    };
    done(ctx, kind, start, Vec::new())
}

fn simple(ctx: &mut Ctx<'_>, kind: Kind) -> PostgresSqlProceduralOccurrence {
    let start = peek_index(ctx).unwrap_or(ctx.index);
    bump(ctx);
    consume_statement(ctx, false);
    done(ctx, kind, start, Vec::new())
}

pub(super) fn rest_unknown(ctx: &mut Ctx<'_>) -> PostgresSqlProceduralOccurrence {
    let start = peek_index(ctx).unwrap_or(ctx.index.saturating_sub(1));
    let nested = collect_keywords_rest(ctx);
    done(ctx, Kind::Unknown, start, nested)
}
