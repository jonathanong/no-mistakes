use super::{PlaceholderPositions, SqlTransformedColumnUseFact};
use crate::codebase::postgres::statements::value::is_placeholder_ident_at;
use sqlparser::ast::{BinaryOperator, Expr, Spanned};
use std::collections::BTreeMap;

pub(super) fn mark(
    expression: &Expr,
    positions: PlaceholderPositions<'_>,
    uses: &mut [SqlTransformedColumnUseFact],
) {
    let mut by_span = BTreeMap::<_, BTreeMap<usize, Vec<usize>>>::new();
    for (index, use_) in uses.iter().enumerate() {
        if use_.clause == super::super::Clause::Having {
            continue;
        }
        by_span
            .entry(span_key(&use_.expression))
            .or_default()
            .entry(std::sync::Arc::as_ptr(&use_.expression) as usize)
            .or_default()
            .push(index);
    }
    positive_predicates(expression, &mut |expression| {
        if let Expr::BinaryOp {
            left,
            op:
                BinaryOperator::Eq
                | BinaryOperator::Lt
                | BinaryOperator::LtEq
                | BinaryOperator::Gt
                | BinaryOperator::GtEq,
            right,
        } = expression
        {
            for (operand, value) in [(left, right), (right, left)] {
                if !constant(value, positions) {
                    continue;
                }
                let Some(roots) = by_span.get(&span_key(operand)) else {
                    continue;
                };
                for indexes in roots.values() {
                    if same_operand(operand, &uses[indexes[0]].expression) {
                        for &index in indexes {
                            uses[index].index_operand = true;
                        }
                    }
                }
            }
        }
    });
}
fn constant(expression: &Expr, positions: PlaceholderPositions<'_>) -> bool {
    match unnest(expression) {
        Expr::Value(value) => !matches!(value.value, sqlparser::ast::Value::Null),
        Expr::Identifier(ident) => positions.is_some() && is_placeholder_ident_at(ident, positions),
        Expr::Tuple(values) => values.iter().all(|value| constant(value, positions)),
        Expr::Cast { expr, .. } => constant(expr, positions),
        Expr::Function(function) => constant_row(function, positions),
        _ => false,
    }
}
fn unnest(mut expression: &Expr) -> &Expr {
    while let Expr::Nested(inner) = expression {
        expression = inner;
    }
    expression
}

fn same_operand(left: &Expr, right: &Expr) -> bool {
    let (left, right) = (unnest(left), unnest(right));
    left.span().start.line > 0 && left.span() == right.span() && left == right
}

fn span_key(expression: &Expr) -> (u64, u64, u64, u64) {
    let span = unnest(expression).span();
    (
        span.start.line,
        span.start.column,
        span.end.line,
        span.end.column,
    )
}

fn constant_row(function: &sqlparser::ast::Function, positions: PlaceholderPositions<'_>) -> bool {
    use sqlparser::ast::{FunctionArg, FunctionArgExpr, FunctionArguments, ObjectNamePart};
    matches!(function.name.0.as_slice(), [ObjectNamePart::Identifier(name)] if name.quote_style.is_none() && name.value.eq_ignore_ascii_case("row"))
        && matches!(function.parameters, FunctionArguments::None)
        && function.filter.is_none()
        && function.over.is_none()
        && function.within_group.is_empty()
        && matches!(&function.args, FunctionArguments::List(list) if !list.args.is_empty()
            && list.clauses.is_empty()
            && list.duplicate_treatment.is_none()
            && list.args.iter().all(|arg| matches!(arg, FunctionArg::Unnamed(FunctionArgExpr::Expr(value)) if constant(value, positions))))
}

fn positive_predicates(expression: &Expr, visit: &mut impl FnMut(&Expr)) {
    let expression = unnest(expression);
    if let Expr::BinaryOp {
        left,
        op: BinaryOperator::And | BinaryOperator::Or,
        right,
    } = expression
    {
        positive_predicates(left, visit);
        positive_predicates(right, visit);
    } else {
        visit(expression);
    }
}
