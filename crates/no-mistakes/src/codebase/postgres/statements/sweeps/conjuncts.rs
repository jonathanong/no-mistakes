use super::super::value::is_placeholder_ident;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::statements::{SqlConjunctFact, SqlCursorBound};
use sqlparser::ast::{BinaryOperator, Expr, Value, Visit, Visitor};
use std::ops::ControlFlow;

/// The top-level `AND` conjuncts of a WHERE clause.
pub(super) fn of(selection: Option<&Expr>, names: &[String]) -> Vec<SqlConjunctFact> {
    let mut leaves = Vec::new();
    if let Some(selection) = selection {
        flatten(selection, &mut leaves);
    }
    leaves
        .into_iter()
        .map(|leaf| {
            let cursor = cursor(leaf, names);
            SqlConjunctFact {
                text: leaf
                    .to_string()
                    .to_ascii_lowercase()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
                cursor_columns: cursor
                    .as_ref()
                    .map_or_else(Vec::new, |cursor| cursor.columns.clone()),
                cursor_bound: cursor.as_ref().map(|cursor| cursor.bound),
                cursor_optional: cursor.as_ref().is_some_and(|cursor| cursor.optional),
                constant_true: is_constant_true(leaf),
                bind_guard: is_bind_guard(leaf),
            }
        })
        .collect()
}

/// A keyset cursor: the columns compared with a bind, which side of them it bounds, and whether
/// the bind may be NULL to switch the comparison off (`($1 IS NULL OR id > $1)`).
struct Cursor {
    columns: Vec<String>,
    bound: SqlCursorBound,
    optional: bool,
}

/// `TRUE`, or a literal equated with itself (`1 = 1`): it never narrows anything.
fn is_constant_true(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => matches!(value.value, Value::Boolean(true)),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } => match (unwrap_expr(left), unwrap_expr(right)) {
            (Expr::Value(left), Expr::Value(right)) => {
                left.value == right.value
                    && !matches!(left.value, Value::Null | Value::Placeholder(_))
            }
            _ => false,
        },
        _ => false,
    }
}

fn flatten<'a>(expr: &'a Expr, out: &mut Vec<&'a Expr>) {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            flatten(left, out);
            flatten(right, out);
        }
        other => out.push(other),
    }
}

/// A plain column of the relation named by `names` (its table and alias), bare or qualified.
pub(super) fn column(expr: &Expr, names: &[String]) -> Option<String> {
    match unwrap_expr(expr) {
        // A recovered interpolation is a value, not a column.
        Expr::Identifier(ident) if is_placeholder_ident(&ident.value) => None,
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
fn cursor(expr: &Expr, names: &[String]) -> Option<Cursor> {
    match expr {
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
            let (columns, lower) = match compare(left, right, names) {
                Some(columns) => (columns, greater),
                None => (compare(right, left, names)?, !greater),
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
            (Expr::IsNull(probe), other) | (other, Expr::IsNull(probe)) if is_bind(probe) => {
                cursor(other, names).map(|cursor| Cursor {
                    optional: true,
                    ..cursor
                })
            }
            _ => None,
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

fn compare(column_side: &Expr, value_side: &Expr, names: &[String]) -> Option<Vec<String>> {
    if !is_bind(value_side) {
        return None;
    }
    match unwrap_expr(column_side) {
        Expr::Tuple(items) => items.iter().map(|item| column(item, names)).collect(),
        other => column(other, names).map(|column| vec![column]),
    }
}

/// A bind parameter, possibly cast, or a row of them. An interpolation recovered from a template
/// literal (`${after}`) reaches the facts as a `sql_placeholder_N` identifier and is a bind too.
fn is_bind(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => matches!(value.value, Value::Placeholder(_)),
        Expr::Identifier(ident) => is_placeholder_ident(&ident.value),
        Expr::Cast { expr, .. } => is_bind(expr),
        Expr::Tuple(items) => !items.is_empty() && items.iter().all(is_bind),
        _ => false,
    }
}

/// A guard can enable a walk, but cannot select individual rows.
fn is_bind_guard(expr: &Expr) -> bool {
    struct Guard {
        bind: bool,
        row_dependent: bool,
    }
    impl Visitor for Guard {
        type Break = ();
        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            match expr {
                Expr::Value(value) if matches!(value.value, Value::Placeholder(_)) => {
                    self.bind = true
                }
                Expr::Identifier(ident) if is_placeholder_ident(&ident.value) => self.bind = true,
                Expr::Identifier(_)
                | Expr::CompoundIdentifier(_)
                | Expr::Function(_)
                | Expr::Subquery(_)
                | Expr::Exists { .. }
                | Expr::InSubquery { .. } => self.row_dependent = true,
                _ => {}
            }
            ControlFlow::Continue(())
        }
    }
    let mut guard = Guard {
        bind: false,
        row_dependent: false,
    };
    let _ = expr.visit(&mut guard);
    guard.bind && !guard.row_dependent
}
