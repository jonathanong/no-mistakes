use super::{bound, column, flatten, is_bind, Cursor};
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{BinaryOperator, Expr};

/// Match the expanded tuple comparison, retaining exact bind identity in every prefix.
pub(super) fn cursor(expr: &Expr, names: &[String]) -> Option<Cursor> {
    fn arms<'a>(expr: &'a Expr, out: &mut Vec<&'a Expr>) {
        match unwrap_expr(expr) {
            Expr::BinaryOp {
                left,
                op: BinaryOperator::Or,
                right,
            } => {
                arms(left, out);
                arms(right, out);
            }
            other => out.push(other),
        }
    }
    let mut alternatives = Vec::new();
    arms(expr, &mut alternatives);
    let mut keys: Vec<(String, &Expr)> = Vec::new();
    let mut direction = None;
    for (index, arm) in alternatives.into_iter().enumerate() {
        let mut terms = Vec::new();
        flatten(arm, &mut terms);
        if terms.len() != index + 1 {
            return None;
        }
        for (prefix, term) in terms.iter().take(index).enumerate() {
            let Expr::BinaryOp {
                left,
                op: BinaryOperator::Eq,
                right,
            } = unwrap_expr(term)
            else {
                return None;
            };
            if column(left, names).as_ref() != Some(&keys[prefix].0)
                || unwrap_expr(right) != keys[prefix].1
            {
                return None;
            }
        }
        let Expr::BinaryOp { left, op, right } = unwrap_expr(terms[index]) else {
            return None;
        };
        let lower = match op {
            BinaryOperator::Gt => true,
            BinaryOperator::Lt => false,
            _ => return None,
        };
        if direction.is_some_and(|direction| direction != lower) || !is_bind(right) {
            return None;
        }
        direction = Some(lower);
        keys.push((column(left, names)?, unwrap_expr(right)));
    }
    Some(Cursor {
        columns: keys.into_iter().map(|key| key.0).collect(),
        bound: bound(direction?),
        optional: false,
    })
}
