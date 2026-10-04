mod bind_guard;
mod conditions;
mod lexicographic;
mod negated;
mod placeholders;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::statements::{SqlConjunctFact, SqlCursorBound};
use conditions::{flatten, is_constant_true};
use placeholders::is_recovered_placeholder;
use sqlparser::ast::{BinaryOperator, Expr, UnaryOperator, Value};

/// The top-level `AND` conjuncts of a WHERE clause.
pub(super) fn of(
    selection: Option<&Expr>,
    names: &[String],
    order_columns: &[String],
    order_ascending: &[Option<bool>],
    transparent_int4_casts: bool,
    recovered_placeholder_positions: &[(u32, u32)],
) -> Vec<SqlConjunctFact> {
    let mut leaves = Vec::new();
    if let Some(selection) = selection {
        flatten(selection, &mut leaves);
    }
    leaves
        .into_iter()
        .map(|leaf| {
            let cursor = cursor(
                leaf,
                names,
                order_columns,
                order_ascending,
                transparent_int4_casts,
                recovered_placeholder_positions,
            );
            SqlConjunctFact {
                text: crate::codebase::postgres::predicate_normalization::normalize(
                    &leaf.to_string(),
                ),
                cursor_columns: cursor
                    .as_ref()
                    .map_or_else(Vec::new, |cursor| cursor.columns.clone()),
                cursor_bound: cursor.as_ref().map(|cursor| cursor.bound),
                cursor_optional: cursor.as_ref().is_some_and(|cursor| cursor.optional),
                constant_true: is_constant_true(leaf),
                bind_guard: bind_guard::is_bind_guard(leaf, recovered_placeholder_positions),
            }
        })
        .collect()
}

/// A keyset cursor: the columns compared with a bind, which side of them it bounds, and whether
/// the bind may be NULL to switch the comparison off (`($1 IS NULL OR id > $1)`).
pub(super) struct Cursor {
    columns: Vec<String>,
    bound: SqlCursorBound,
    optional: bool,
}

/// A plain column of the relation named by `names` (its table and alias), bare or qualified.
pub(super) fn column(
    expr: &Expr,
    names: &[String],
    recovered_placeholder_positions: &[(u32, u32)],
) -> Option<String> {
    match unwrap_expr(expr) {
        // A recovered interpolation is a value, not a column.
        Expr::Identifier(ident)
            if is_recovered_placeholder(ident, recovered_placeholder_positions) =>
        {
            None
        }
        Expr::Identifier(ident) => Some(ident_key(ident)),
        Expr::CompoundIdentifier(parts) => {
            let (column, qualifier) = (parts.last()?, parts.get(parts.len().checked_sub(2)?)?);
            names
                .contains(&ident_key(qualifier))
                .then(|| ident_key(column))
        }
        _ => None,
    }
}

/// The ORDER BY-style columns a conjunct compares with a bind parameter, or none when it is not
/// a cursor.
fn cursor(
    expr: &Expr,
    names: &[String],
    order_columns: &[String],
    order_ascending: &[Option<bool>],
    transparent_int4_casts: bool,
    recovered_placeholder_positions: &[(u32, u32)],
) -> Option<Cursor> {
    match unwrap_expr(expr) {
        Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr,
        } => negated::of(expr, |inner| {
            cursor(
                inner,
                names,
                order_columns,
                order_ascending,
                transparent_int4_casts,
                recovered_placeholder_positions,
            )
        }),

        Expr::BinaryOp {
            left,
            op:
                op @ (BinaryOperator::Gt
                | BinaryOperator::GtEq
                | BinaryOperator::Lt
                | BinaryOperator::LtEq),
            right,
        } => {
            let greater = matches!(op, BinaryOperator::Gt | BinaryOperator::GtEq);
            // `id > $1` bounds id from below; `$1 > id` bounds it from above.
            let (columns, lower) =
                match compare(left, right, names, recovered_placeholder_positions) {
                    Some(columns) => (columns, greater),
                    None => (
                        compare(right, left, names, recovered_placeholder_positions)?,
                        !greater,
                    ),
                };
            Some(Cursor {
                columns,
                bound: bound(lower),
                optional: false,
            })
        }
        // `($1 IS NULL OR id > $1)`, in either order: the cursor is optional.
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Or,
            right,
        } => match (unwrap_expr(left), unwrap_expr(right)) {
            (Expr::IsNull(probe), other) | (other, Expr::IsNull(probe))
                if is_bind(probe, recovered_placeholder_positions) =>
            {
                cursor(
                    other,
                    names,
                    order_columns,
                    order_ascending,
                    transparent_int4_casts,
                    recovered_placeholder_positions,
                )
                .map(|cursor| Cursor {
                    optional: true,
                    ..cursor
                })
            }
            _ => lexicographic::cursor(
                expr,
                names,
                order_columns,
                order_ascending,
                transparent_int4_casts,
                recovered_placeholder_positions,
            ),
        },
        _ => None,
    }
}

fn bound(lower: bool) -> SqlCursorBound {
    if lower {
        SqlCursorBound::Lower
    } else {
        SqlCursorBound::Upper
    }
}

fn compare(
    column_side: &Expr,
    value_side: &Expr,
    names: &[String],
    recovered_placeholder_positions: &[(u32, u32)],
) -> Option<Vec<String>> {
    if !is_bind(value_side, recovered_placeholder_positions) {
        return None;
    }
    match unwrap_expr(column_side) {
        Expr::Tuple(items) => items
            .iter()
            .map(|item| column(item, names, recovered_placeholder_positions))
            .collect(),
        other => column(other, names, recovered_placeholder_positions).map(|column| vec![column]),
    }
}

/// A bind parameter, possibly cast, or a row of them. An interpolation recovered from a template
/// literal (`${after}`) reaches the facts as a `sql_placeholder_N` identifier and is a bind too.
fn is_bind(expr: &Expr, recovered_placeholder_positions: &[(u32, u32)]) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => matches!(value.value, Value::Placeholder(_)),
        Expr::Identifier(ident) => is_recovered_placeholder(ident, recovered_placeholder_positions),
        Expr::Cast { expr, .. } => is_bind(expr, recovered_placeholder_positions),
        Expr::Tuple(items) => {
            !items.is_empty()
                && items
                    .iter()
                    .all(|item| is_bind(item, recovered_placeholder_positions))
        }
        _ => false,
    }
}
