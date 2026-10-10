//! Delimiters come from the expression's original prepared tokens, never a reparse.
use crate::codebase::postgres::source::{expressions, locations::Locations, types::*};
use sqlparser::{
    ast::{Expr, Spanned},
    tokenizer::{Span, Token, TokenWithSpan},
};

pub(super) fn delimiters(tokens: &[TokenWithSpan]) -> Vec<Span> {
    let mut stack = Vec::new();
    let mut delimiters = Vec::new();
    for token in tokens {
        match token.token {
            Token::LParen | Token::LBracket => stack.push(token.span.start),
            Token::RParen | Token::RBracket => {
                if let Some(start) = stack.pop() {
                    delimiters.push(Span {
                        start,
                        end: token.span.end,
                    });
                }
            }
            _ => {}
        }
    }
    delimiters
}

pub(super) fn locate_delimiters(
    delimiters: &[Span],
    locations: &Locations<'_>,
) -> Vec<PostgresSqlSpan> {
    let mut prepared = delimiters
        .iter()
        .filter_map(|span| locations.span(*span))
        .collect::<Vec<_>>();
    prepared.sort_by_key(|span| span.start.offset);
    prepared
}

pub(super) fn source_expression(
    expr: &Expr,
    delimiters: &[PostgresSqlSpan],
    locations: &Locations<'_>,
) -> PostgresSqlExpression {
    let mut projected = expression_prepared(expr, expr.span(), delimiters, locations);
    if projected.children_complete && projected.children.iter().all(|child| child.span.is_some()) {
        projected.span =
            super::super::expression_children::exact_ast_span(expr, locations, delimiters);
    }
    projected.span = locations.span_covering(projected.span, &projected.sql);
    projected
}

pub(super) fn expression(
    expr: &Expr,
    span: Span,
    delimiters: &[Span],
    locations: &Locations<'_>,
) -> PostgresSqlExpression {
    let delimiters = locate_delimiters(delimiters, locations);
    expression_prepared(expr, span, &delimiters, locations)
}

fn expression_prepared(
    expr: &Expr,
    span: Span,
    delimiters: &[PostgresSqlSpan],
    locations: &Locations<'_>,
) -> PostgresSqlExpression {
    let mut projected = expressions::expression_with_delimiters(expr, locations, delimiters);
    for function in &mut projected.functions {
        repair(&mut function.span, delimiters);
    }
    arguments(&mut projected.root, delimiters, locations);
    child_spans(&mut projected.children, delimiters, locations);
    projected.span = locations.span(span);
    repair(&mut projected.span, delimiters);
    projected.span = locations.span_covering(projected.span, &projected.sql);
    projected
}

fn child_spans(
    children: &mut [PostgresSqlExpressionChild],
    delimiters: &[PostgresSqlSpan],
    locations: &Locations<'_>,
) {
    for child in children {
        repair(&mut child.span, delimiters);
        child.span = locations.span_covering(child.span.clone(), &child.sql);
        child_spans(&mut child.children, delimiters, locations);
    }
}

fn repair(span: &mut Option<PostgresSqlSpan>, delimiters: &[PostgresSqlSpan]) {
    let Some(span) = span else { return };
    let start = delimiters.partition_point(|delimiter| delimiter.start.offset < span.start.offset);
    for delimiter in &delimiters[start..] {
        if delimiter.start.offset <= span.end.offset && delimiter.end.offset > span.end.offset {
            span.end = delimiter.end.clone();
        } else if delimiter.start.offset > span.end.offset {
            break;
        }
    }
}

fn arguments(
    root: &mut PostgresSqlExpressionRoot,
    delimiters: &[PostgresSqlSpan],
    locations: &Locations<'_>,
) {
    match root {
        PostgresSqlExpressionRoot::FunctionCall {
            arguments: args, ..
        } => {
            for argument in args {
                repair(&mut argument.span, delimiters);
                argument.span = locations.span_covering(argument.span.clone(), &argument.sql);
                arguments(&mut argument.root, delimiters, locations);
            }
        }
        PostgresSqlExpressionRoot::Parenthesized { expression }
        | PostgresSqlExpressionRoot::Cast { expression, .. }
        | PostgresSqlExpressionRoot::Unary { expression, .. } => {
            arguments(expression, delimiters, locations)
        }
        _ => {}
    }
}
