use super::SqlTransformedColumnUseFact;
use sqlparser::ast::{BinaryOperator, Expr, Spanned};
use std::collections::BTreeMap;

pub(super) fn mark(expression: &Expr, uses: &mut [SqlTransformedColumnUseFact]) {
    let mut by_span = BTreeMap::<_, BTreeMap<usize, Vec<usize>>>::new();
    for (index, use_) in uses.iter().enumerate() {
        by_span
            .entry(span_key(&use_.expression))
            .or_default()
            .entry(std::sync::Arc::as_ptr(&use_.expression) as usize)
            .or_default()
            .push(index);
    }
    super::super::scope::visit_expressions(expression, &mut |expression| {
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
                if !constant(value) {
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
fn constant(expression: &Expr) -> bool {
    match unnest(expression) {
        Expr::Value(_) => true,
        Expr::Tuple(values) => values.iter().all(constant),
        Expr::Cast { expr, .. } => constant(expr),
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
