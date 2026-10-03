use super::{bound, column, flatten, is_bind, is_placeholder_ident, Cursor};
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{BinaryOperator, DataType, Expr, Value};

/// Match the expanded tuple comparison, retaining bind identity in every prefix.
pub(super) fn cursor(
    expr: &Expr,
    names: &[String],
    order_columns: &[String],
    order_ascending: &[Option<bool>],
    transparent_int4_casts: bool,
) -> Option<Cursor> {
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
    let mut alternatives: Vec<_> = alternatives
        .into_iter()
        .map(|arm| {
            let mut terms = Vec::new();
            flatten(arm, &mut terms);
            terms
        })
        .collect();
    // The number of equality-prefix terms identifies an arm's position in the
    // expanded cursor. OR order is semantically irrelevant, so normalize it
    // before checking the complete prefix chain.
    alternatives.sort_by_key(Vec::len);
    let mut keys: Vec<(String, &Expr)> = Vec::new();
    let mut direction = None;
    for (index, terms) in alternatives.into_iter().enumerate() {
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
            let (column_expr, bind_expr) = match (
                column(left, names).is_some() && is_bind(right),
                column(right, names).is_some() && is_bind(left),
            ) {
                (true, false) => (left, right),
                (false, true) => (right, left),
                _ => return None,
            };
            if column(column_expr, names).as_ref() != Some(&keys[prefix].0)
                || bind_identity(bind_expr, transparent_int4_casts)? != keys[prefix].1
            {
                return None;
            }
        }
        let Expr::BinaryOp { left, op, right } = unwrap_expr(terms[index]) else {
            return None;
        };
        let (column_expr, bind_expr, lower) = match (
            column(left, names).is_some() && is_bind(right),
            column(right, names).is_some() && is_bind(left),
        ) {
            (true, false) => match op {
                BinaryOperator::Gt => (left, right, true),
                BinaryOperator::Lt => (left, right, false),
                _ => return None,
            },
            (false, true) => match op {
                BinaryOperator::Gt => (right, left, false),
                BinaryOperator::Lt => (right, left, true),
                _ => return None,
            },
            _ => return None,
        };
        if direction.is_some_and(|direction| direction != lower) {
            return None;
        }
        direction = Some(lower);
        keys.push((
            column(column_expr, names)?,
            bind_identity(bind_expr, transparent_int4_casts)?,
        ));
    }
    // An expanded comparison is contiguous only in its ORDER BY key sequence. Mixed sort
    // directions need different range operators per arm, which this matcher does not accept.
    if !keys
        .iter()
        .map(|key| &key.0)
        .eq(order_columns.iter().take(keys.len()))
        || order_ascending
            .iter()
            .take(keys.len())
            .any(|ascending| *ascending != order_ascending[0] || ascending.is_none())
    {
        return None;
    }
    Some(Cursor {
        columns: keys.into_iter().map(|key| key.0).collect(),
        bound: bound(direction?),
        optional: false,
    })
}

/// Keep an int4 bind's identity through built-in transparent spellings. Other casts retain
/// their complete expression as an opaque identity, preserving an identical-bound comparison.
fn bind_identity(expr: &Expr, transparent_int4_casts: bool) -> Option<&Expr> {
    match unwrap_expr(expr) {
        Expr::Value(value) if matches!(value.value, Value::Placeholder(_)) => {
            Some(unwrap_expr(expr))
        }
        Expr::Identifier(ident) if is_placeholder_ident(&ident.value) => Some(unwrap_expr(expr)),
        Expr::Tuple(items) if !items.is_empty() && items.iter().all(is_bind) => {
            Some(unwrap_expr(expr))
        }
        Expr::Cast {
            expr: inner,
            data_type,
            ..
        } if transparent_int4_casts && transparent_int4_cast(data_type) => {
            let identity = bind_identity(inner, transparent_int4_casts)?;
            if matches!(identity, Expr::Cast { .. }) {
                Some(unwrap_expr(expr))
            } else {
                Some(identity)
            }
        }
        Expr::Cast { .. } if is_bind(expr) => Some(unwrap_expr(expr)),
        _ => None,
    }
}

fn transparent_int4_cast(data_type: &DataType) -> bool {
    matches!(
        data_type,
        DataType::Int(_) | DataType::Int4(_) | DataType::Integer(_)
    )
}
