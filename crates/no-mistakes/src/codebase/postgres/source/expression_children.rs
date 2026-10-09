mod function;
mod roots;
#[cfg(test)]
mod tests;

use super::{locations::Locations, types::*};
use sqlparser::ast::Expr;

pub(super) fn project_with_delimiters(
    expr: &Expr,
    locations: &Locations<'_>,
    delimiters: &[PostgresSqlSpan],
) -> (Vec<PostgresSqlExpressionChild>, bool) {
    children(expr, locations, delimiters, 0)
}

struct Spec<'a> {
    role: PostgresSqlExpressionChildRole,
    index: Option<usize>,
    argument_name: Option<PostgresSqlIdentifier>,
    expr: &'a Expr,
}

fn children(
    expr: &Expr,
    locations: &Locations<'_>,
    delimiters: &[PostgresSqlSpan],
    depth: usize,
) -> (Vec<PostgresSqlExpressionChild>, bool) {
    if depth >= 64 {
        return (Vec::new(), false);
    }
    let (specs, mut complete) = specs(expr);
    let mut projected = Vec::with_capacity(specs.len());
    for spec in specs {
        let (nested, nested_complete) = children(spec.expr, locations, delimiters, depth + 1);
        let span = if nested_complete && nested.iter().all(|child| child.span.is_some()) {
            exact_ast_span(spec.expr, locations, delimiters)
        } else {
            None
        };
        // Structural coverage is independent of nullable source provenance.
        complete &= nested_complete;
        projected.push(PostgresSqlExpressionChild {
            role: spec.role,
            index: spec.index,
            argument_name: spec.argument_name,
            sql: spec.expr.to_string(),
            span,
            root: roots::root(spec.expr, locations),
            children: nested,
            children_complete: nested_complete,
        });
    }
    (projected, complete)
}

pub(super) fn exact_ast_span(
    expr: &Expr,
    locations: &Locations<'_>,
    delimiters: &[PostgresSqlSpan],
) -> Option<PostgresSqlSpan> {
    if matches!(
        expr,
        Expr::Nested(_)
            | Expr::UnaryOp { .. }
            | Expr::Cast { .. }
            | Expr::TypedString { .. }
            | Expr::IsNull(_)
            | Expr::IsNotNull(_)
    ) {
        return None;
    }
    let mut span = locations.span(sqlparser::ast::Spanned::span(expr))?;
    let start = delimiters.partition_point(|delimiter| delimiter.start.offset < span.start.offset);
    if let Expr::Function(function) = expr {
        if matches!(function.args, sqlparser::ast::FunctionArguments::List(_)) {
            if let Some(delimiter) = delimiters[start..]
                .iter()
                .find(|delimiter| delimiter.end.offset >= span.end.offset)
            {
                span.end = delimiter.end.clone();
            } else {
                return None;
            }
        }
    }
    for delimiter in &delimiters[start..] {
        if delimiter.start.offset <= span.end.offset && delimiter.end.offset > span.end.offset {
            span.end = delimiter.end.clone();
        } else if delimiter.start.offset > span.end.offset {
            break;
        }
    }
    Some(span)
}

fn specs(expr: &Expr) -> (Vec<Spec<'_>>, bool) {
    use PostgresSqlExpressionChildRole as Role;
    match expr {
        Expr::BinaryOp { left, right, .. } => (
            vec![
                spec(Role::BinaryLeft, None, left),
                spec(Role::BinaryRight, None, right),
            ],
            true,
        ),
        Expr::IsNull(expr) | Expr::IsNotNull(expr) => {
            (vec![spec(Role::NullOperand, None, expr)], true)
        }
        Expr::IsDistinctFrom(left, right) | Expr::IsNotDistinctFrom(left, right) => (
            vec![
                spec(Role::DistinctLeft, None, left),
                spec(Role::DistinctRight, None, right),
            ],
            true,
        ),
        Expr::TypedString(_) => (Vec::new(), true),
        Expr::UnaryOp { expr, .. } => (vec![spec(Role::UnaryOperand, None, expr)], true),
        Expr::Nested(expr) => (vec![spec(Role::ParenthesizedExpression, None, expr)], true),
        Expr::Cast {
            kind, expr, format, ..
        } => (
            vec![spec(Role::CastOperand, None, expr)],
            matches!(
                kind,
                sqlparser::ast::CastKind::Cast | sqlparser::ast::CastKind::DoubleColon
            ) && format.is_none(),
        ),
        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => {
            let mut children = Vec::with_capacity(
                usize::from(operand.is_some())
                    + conditions.len() * 2
                    + usize::from(else_result.is_some()),
            );
            if let Some(expr) = operand {
                children.push(spec(Role::CaseOperand, None, expr));
            }
            for (index, condition) in conditions.iter().enumerate() {
                children.push(spec(
                    Role::CaseWhenCondition,
                    Some(index),
                    &condition.condition,
                ));
                children.push(spec(Role::CaseWhenResult, Some(index), &condition.result));
            }
            if let Some(expr) = else_result {
                children.push(spec(Role::CaseElse, None, expr));
            }
            (children, true)
        }
        Expr::Function(function) => function::specs(function),
        Expr::Identifier(_) | Expr::CompoundIdentifier(_) | Expr::Value(_) => (Vec::new(), true),
        _ => (Vec::new(), false),
    }
}

fn spec(role: PostgresSqlExpressionChildRole, index: Option<usize>, expr: &Expr) -> Spec<'_> {
    Spec {
        role,
        index,
        argument_name: None,
        expr,
    }
}
