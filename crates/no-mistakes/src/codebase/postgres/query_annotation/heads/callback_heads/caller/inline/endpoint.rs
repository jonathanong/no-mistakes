//! Cheap endpoint shape shared by preflight and the complete parameter census.
use super::{options, Span};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{CallExpression, Expression};

pub(super) fn matching(
    call: &CallExpression<'_>,
    name: &str,
    executors: &FxHashSet<Span>,
) -> Option<(Span, bool)> {
    let span = (call.span.start, call.span.end);
    if call.optional
        || call.arguments.len() > 2
        || !executors.contains(&span)
        || call
            .arguments
            .iter()
            .any(|arg| arg.as_expression().is_none())
    {
        return None;
    }
    let Expression::Identifier(arg) = unwrap_ts_wrappers(call.arguments.first()?.as_expression()?)
    else {
        return None;
    };
    if arg.name != name {
        return None;
    }
    let needs_fresh = call.arguments.len() == 2;
    if needs_fresh && !options(call.arguments[1].as_expression()?) {
        return None;
    }
    Some((span, needs_fresh))
}
