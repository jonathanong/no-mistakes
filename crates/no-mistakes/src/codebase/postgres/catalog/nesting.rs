//! Redundant-parenthesis removal for boolean `AND`/`OR` chains.
//!
//! `pg_get_expr` wraps every conjunct, `((a IS NULL) AND (b IS NOT NULL))`, while hand-written
//! predicates do not. Parentheses are dropped only where precedence makes them redundant, so
//! `(a OR b) AND c` never collapses into `a OR b AND c`. Operand order is preserved.
use sqlparser::ast::{BinaryOperator, Expr};

pub(super) fn strip_redundant_nesting(expression: Expr) -> Expr {
    let expression = unwrap_nested(expression);
    match expression {
        Expr::BinaryOp { left, op, right } if is_boolean_chain(&op) => {
            let left = Box::new(operand(*left, &op));
            let right = Box::new(operand(*right, &op));
            Expr::BinaryOp { left, op, right }
        }
        other => other,
    }
}

fn operand(expression: Expr, parent: &BinaryOperator) -> Expr {
    let inner = strip_redundant_nesting(expression);
    // Only an `OR` nested under `AND` changes meaning without its parentheses.
    match (&inner, parent) {
        (Expr::BinaryOp { op: BinaryOperator::Or, .. }, BinaryOperator::And) => {
            Expr::Nested(Box::new(inner))
        }
        _ => inner,
    }
}

fn is_boolean_chain(op: &BinaryOperator) -> bool {
    matches!(op, BinaryOperator::And | BinaryOperator::Or)
}

fn unwrap_nested(mut expression: Expr) -> Expr {
    while let Expr::Nested(inner) = expression {
        expression = *inner;
    }
    expression
}
