use super::unwrap_expr;
use sqlparser::ast::{Assignment, Expr, Value};

pub(super) fn plain_value(expr: &Expr) -> Option<&Value> {
    match unwrap_expr(expr) {
        Expr::Value(value)
            if matches!(
                value.value,
                Value::Number(..)
                    | Value::SingleQuotedString(_)
                    | Value::Boolean(_)
                    | Value::Placeholder(_)
            ) =>
        {
            Some(&value.value)
        }
        _ => None,
    }
}

pub(super) fn plain_assignments(assignments: &[Assignment]) -> bool {
    assignments.iter().all(|assignment| {
        matches!(
            unwrap_expr(&assignment.value),
            Expr::Identifier(_) | Expr::CompoundIdentifier(_)
        ) || plain_value(&assignment.value).is_some()
    })
}
