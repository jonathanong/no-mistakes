use super::cursor::{self, at_plain_end, at_word, eat_word, peek_index, OpenLabel};
use super::literal::command_kind;
use super::scan::{self, consume_statement};
use super::{done, walk_statements, Ctx, Stop};
use crate::codebase::postgres::source::types::{
    PostgresSqlProceduralOccurrence, PostgresSqlProceduralOccurrenceKind as Kind,
};

pub(super) fn if_stmt(ctx: &mut Ctx<'_>) -> PostgresSqlProceduralOccurrence {
    let start = peek_index(ctx).unwrap_or(ctx.index);
    eat_word(ctx, "IF");
    let mut nested = Vec::new();
    let mut empty = false;
    loop {
        // Peek before `scan_header` consumes the condition. ELSE has no header.
        empty |= empty_header(ctx, "THEN");
        nested.extend(scan::scan_header(ctx, &["THEN"]));
        if !eat_word(ctx, "THEN") {
            return done(ctx, Kind::Unknown, start, nested);
        }
        nested.extend(walk_statements(ctx, Stop::IfBranch));
        if eat_word(ctx, "ELSIF") || eat_word(ctx, "ELSEIF") {
            continue;
        }
        if eat_word(ctx, "ELSE") {
            nested.extend(walk_statements(ctx, Stop::IfBranch));
        }
        break;
    }
    if eat_word(ctx, "END") && eat_word(ctx, "IF") {
        cursor::eat_semi(ctx);
        done(
            ctx,
            if empty {
                Kind::Unknown
            } else {
                Kind::ControlFlow
            },
            start,
            nested,
        )
    } else {
        done(ctx, Kind::Unknown, start, nested)
    }
}

pub(super) fn case_stmt(ctx: &mut Ctx<'_>) -> PostgresSqlProceduralOccurrence {
    let start = peek_index(ctx).unwrap_or(ctx.index);
    eat_word(ctx, "CASE");
    ctx.walker_only = true;
    let mut nested = scan::scan_header(ctx, &["WHEN", "ELSE", "END"]);
    while eat_word(ctx, "WHEN") {
        nested.extend(scan::scan_header(ctx, &["THEN"]));
        if !eat_word(ctx, "THEN") {
            return done(ctx, Kind::Unknown, start, nested);
        }
        nested.extend(walk_statements(ctx, Stop::CaseBranch));
    }
    if eat_word(ctx, "ELSE") {
        nested.extend(walk_statements(ctx, Stop::CaseBranch));
    }
    if eat_word(ctx, "END") && eat_word(ctx, "CASE") {
        cursor::eat_semi(ctx);
        done(ctx, Kind::ControlFlow, start, nested)
    } else {
        done(ctx, Kind::Unknown, start, nested)
    }
}

pub(super) fn loop_stmt(ctx: &mut Ctx<'_>, label: OpenLabel) -> PostgresSqlProceduralOccurrence {
    let start = peek_index(ctx).unwrap_or(ctx.index);
    let bare = at_word(ctx, "LOOP");
    cursor::bump(ctx);
    ctx.walker_only = true;
    let mut nested = Vec::new();
    if !bare {
        // Same peek as IF. Closing-label checks stay on the non-empty path below.
        let empty = empty_header(ctx, "LOOP");
        nested.extend(scan::scan_header(ctx, &["LOOP"]));
        if !eat_word(ctx, "LOOP") {
            return done(ctx, Kind::Unknown, start, nested);
        }
        if empty {
            return finish_unknown_loop(ctx, start, nested, &label);
        }
    }
    nested.extend(walk_statements(ctx, Stop::Loop));
    if eat_word(ctx, "END") && eat_word(ctx, "LOOP") {
        let close = cursor::eat_label(ctx);
        cursor::note_label(ctx, &label, close.as_ref());
        cursor::eat_semi(ctx);
        done(ctx, Kind::ControlFlow, start, nested)
    } else {
        done(ctx, Kind::Unknown, start, nested)
    }
}

/// A conditional header with no token before its stop word. Whitespace is skipped.
fn empty_header(ctx: &Ctx<'_>, stop: &str) -> bool {
    at_word(ctx, stop)
}

/// Walk the body and the closing `END LOOP` so later statements stay aligned.
fn finish_unknown_loop(
    ctx: &mut Ctx<'_>,
    start: usize,
    mut nested: Vec<PostgresSqlProceduralOccurrence>,
    label: &OpenLabel,
) -> PostgresSqlProceduralOccurrence {
    nested.extend(walk_statements(ctx, Stop::Loop));
    if eat_word(ctx, "END") && eat_word(ctx, "LOOP") {
        let close = cursor::eat_label(ctx);
        cursor::note_label(ctx, label, close.as_ref());
        cursor::eat_semi(ctx);
    }
    done(ctx, Kind::Unknown, start, nested)
}

pub(super) fn begin_stmt(ctx: &mut Ctx<'_>, label: OpenLabel) -> PostgresSqlProceduralOccurrence {
    let start = peek_index(ctx).unwrap_or(ctx.index);
    eat_word(ctx, "BEGIN");
    let mut nested = walk_statements(ctx, Stop::Outer);
    if at_word(ctx, "EXCEPTION") {
        let exception_at = peek_index(ctx).unwrap_or(start);
        eat_word(ctx, "EXCEPTION");
        ctx.walker_only = true;
        nested.push(exception_handler(ctx, exception_at));
    }
    if at_plain_end(ctx) {
        eat_word(ctx, "END");
        let close = cursor::eat_label(ctx);
        cursor::note_label(ctx, &label, close.as_ref());
        cursor::eat_semi(ctx);
        done(ctx, Kind::ControlFlow, start, nested)
    } else {
        done(ctx, Kind::Unknown, start, nested)
    }
}

pub(super) fn exception(ctx: &mut Ctx<'_>, start: usize) -> PostgresSqlProceduralOccurrence {
    exception_handler(ctx, start)
}

fn exception_handler(ctx: &mut Ctx<'_>, start: usize) -> PostgresSqlProceduralOccurrence {
    let mut nested = Vec::new();
    let mut saw_when = false;
    while eat_word(ctx, "WHEN") {
        saw_when = true;
        nested.extend(scan::scan_header(ctx, &["THEN"]));
        if !eat_word(ctx, "THEN") {
            return done(ctx, Kind::Unknown, start, nested);
        }
        nested.extend(walk_statements(ctx, Stop::Handler));
    }
    // A bare EXCEPTION has no handler. A WHEN arm that lacks THEN already
    // returned Unknown above, so only a consumed arm is control flow.
    let kind = if saw_when {
        Kind::ControlFlow
    } else {
        Kind::Unknown
    };
    done(ctx, kind, start, nested)
}

pub(super) fn raise_stmt(ctx: &mut Ctx<'_>) -> PostgresSqlProceduralOccurrence {
    let start = peek_index(ctx).unwrap_or(ctx.index);
    eat_word(ctx, "RAISE");
    ctx.walker_only = true;
    let nested = scan::collect_keywords_until_semi(ctx);
    done(ctx, Kind::ControlFlow, start, nested)
}

pub(super) fn execute_stmt(ctx: &mut Ctx<'_>) -> PostgresSqlProceduralOccurrence {
    let start = peek_index(ctx).unwrap_or(ctx.index);
    eat_word(ctx, "EXECUTE");
    let command = scan::take_command(ctx);
    consume_statement(ctx, false);
    let kind = command_kind(ctx.tokens, &command, ctx.depth);
    done(ctx, kind, start, Vec::new())
}
