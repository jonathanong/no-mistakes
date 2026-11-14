use super::SqlTransformedColumnUseFact;
use sqlparser::ast::{BinaryOperator, Expr, Spanned};

pub(super) fn mark(expression: &Expr, uses: &mut [SqlTransformedColumnUseFact]) {
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
                if constant(value) {
                    for use_ in uses
                        .iter_mut()
                        .filter(|use_| same_operand(operand, &use_.expression))
                    {
                        use_.index_operand = true;
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
