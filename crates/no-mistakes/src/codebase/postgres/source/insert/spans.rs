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
    // A partial parser span is not provenance. Comments and whitespace are still
    // the expression's source bytes; a different token spelling is not.
    if projected
        .span
        .as_ref()
        .is_some_and(|span| !source_matches_rendered(locations.slice(span), &projected.sql))
    {
        projected.span = None;
    }
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

/// `now /*keep*/ ()` matches rendered `now()`. `integer` does not match `INTEGER`.
fn source_matches_rendered(source: &str, rendered: &str) -> bool {
    let mut source = source.chars().peekable();
    let mut rendered = rendered.chars().peekable();
    loop {
        while rendered.peek().is_some_and(|c| c.is_whitespace()) {
            rendered.next();
        }
        while source.peek().is_some_and(|c| c.is_whitespace()) {
            source.next();
        }
        if starts_with(&source, &['/', '*']) {
            source.next();
            source.next();
            let mut depth = 1;
            while depth > 0 {
                match source.next() {
                    Some('/') if source.peek() == Some(&'*') => {
                        source.next();
                        depth += 1;
                    }
                    Some('*') if source.peek() == Some(&'/') => {
                        source.next();
                        depth -= 1;
                    }
                    Some(_) => {}
                    None => return false,
                }
            }
            continue;
        }
        if starts_with(&source, &['-', '-']) {
            source.next();
            source.next();
            while source.peek().is_some_and(|c| *c != '\n') {
                source.next();
            }
            continue;
        }
        match (source.next(), rendered.next()) {
            (None, None) => return true,
            (Some(left), Some(right)) if left == right => {}
            _ => return false,
        }
    }
}

fn starts_with(chars: &std::iter::Peekable<std::str::Chars<'_>>, prefix: &[char]) -> bool {
    chars.clone().take(prefix.len()).eq(prefix.iter().copied())
}

#[cfg(test)]
mod tests;
