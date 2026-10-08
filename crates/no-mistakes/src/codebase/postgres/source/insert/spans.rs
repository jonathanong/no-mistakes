//! Delimiters come from the expression's original prepared tokens, never a reparse.
use crate::codebase::postgres::source::{expressions, locations::Locations, types::*};
use sqlparser::{ast::Expr, tokenizer::Span};

pub(super) fn expression(
    expr: &Expr,
    span: Span,
    delimiters: &[Span],
    locations: &Locations<'_>,
) -> PostgresSqlExpression {
    let mut projected = expressions::expression(expr, locations);
    let delimiters = delimiters
        .iter()
        .filter_map(|span| locations.span(*span))
        .collect::<Vec<_>>();
    for function in &mut projected.functions {
        repair(&mut function.span, &delimiters);
    }
    arguments(&mut projected.root, &delimiters);
    projected.span = locations.span(span);
    projected
}

fn repair(span: &mut Option<PostgresSqlSpan>, delimiters: &[PostgresSqlSpan]) {
    let Some(span) = span else { return };
    let end = span.end.offset;
    for delimiter in delimiters {
        if delimiter.start.offset >= span.start.offset
            && delimiter.start.offset <= end
            && delimiter.end.offset > span.end.offset
        {
            span.end = delimiter.end.clone();
        }
    }
}

fn arguments(root: &mut PostgresSqlExpressionRoot, delimiters: &[PostgresSqlSpan]) {
    match root {
        PostgresSqlExpressionRoot::FunctionCall {
            arguments: args, ..
        } => {
            for argument in args {
                repair(&mut argument.span, delimiters);
                arguments(&mut argument.root, delimiters);
            }
        }
        PostgresSqlExpressionRoot::Parenthesized { expression }
        | PostgresSqlExpressionRoot::Cast { expression, .. } => arguments(expression, delimiters),
        _ => {}
    }
}
