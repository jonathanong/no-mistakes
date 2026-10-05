//! Pin evidence for a single-relation `INSERT ... SELECT` and the constant-value test shared with
//! the conflict-ordering rule.
use super::single_row::{is_bind, projection_expr, single_valued};
use super::SqlConflictInsertFact;
use crate::codebase::postgres::parse_postgres_expression;
use sqlparser::ast::{BinaryOperator, Expr, Query, SetExpr, TableFactor, With};

/// A single plain relation whose WHERE equates columns to constants, so a catalog unique key
/// among those columns bounds the source to one row.
pub(super) fn pinned_relation(query: &Query, binds: &[(u32, u32)]) -> Option<SqlPinnedRelation> {
    if query.with.is_some() {
        return None;
    }
    let SetExpr::Select(select) = query.body.as_ref() else {
        return None;
    };
    let [from] = select.from.as_slice() else {
        return None;
    };
    let TableFactor::Table {
        name,
        alias,
        args: None,
        ..
    } = &from.relation
    else {
        return None;
    };
    if !from.joins.is_empty()
        || !select
            .projection
            .iter()
            .all(|item| projection_expr(item).is_some_and(|expr| single_valued(expr, true, binds)))
    {
        return None;
    }
    let qualifier = alias
        .as_ref()
        .map(|alias| alias.name.value.clone())
        .or_else(|| {
            name.0
                .last()
                .map(|part| part.to_string().trim_matches('"').to_owned())
        })?
        .to_ascii_lowercase();
    let mut columns = Vec::new();
    collect_pins(select.selection.as_ref()?, &qualifier, binds, &mut columns);
    (!columns.is_empty()).then(|| SqlPinnedRelation {
        table: name.to_string(),
        columns,
    })
}

fn collect_pins(expr: &Expr, qualifier: &str, binds: &[(u32, u32)], columns: &mut Vec<String>) {
    match expr {
        Expr::Nested(inner) => collect_pins(inner, qualifier, binds, columns),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            collect_pins(left, qualifier, binds, columns);
            collect_pins(right, qualifier, binds, columns);
        }
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } => {
            for (column, other) in [(left, right), (right, left)] {
                if let Some(column) =
                    column_of(column, qualifier, binds).filter(|_| expr_is_constant(other, binds))
                {
                    columns.push(column);
                }
            }
        }
        _ => {}
    }
}

fn column_of(expr: &Expr, qualifier: &str, binds: &[(u32, u32)]) -> Option<String> {
    match expr {
        Expr::Nested(inner) => column_of(inner, qualifier, binds),
        Expr::Identifier(ident) if !is_bind(ident, binds) => Some(ident.to_string()),
        Expr::CompoundIdentifier(parts) => match parts.as_slice() {
            [owner, column] if owner.value.eq_ignore_ascii_case(qualifier) => {
                Some(column.to_string())
            }
            _ => None,
        },
        _ => None,
    }
}

fn expr_is_constant(expr: &Expr, binds: &[(u32, u32)]) -> bool {
    match expr {
        Expr::Value(_) => true,
        Expr::Identifier(ident) => is_bind(ident, binds),
        Expr::Nested(inner)
        | Expr::Cast { expr: inner, .. }
        | Expr::UnaryOp { expr: inner, .. } => expr_is_constant(inner, binds),
        _ => false,
    }
}

/// Whether `expression` is a literal or bound parameter (possibly cast), so it has the same
/// value in every row and cannot change the order of rows.
pub fn expression_is_constant(expression: &str) -> bool {
    parse_postgres_expression(expression).is_some_and(|expr| expr_is_constant(&expr, &[]))
}

/// A single plain relation whose `WHERE` equates `columns` to constants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlPinnedRelation {
    pub table: String,
    pub columns: Vec<String>,
}

/// Drop pin evidence for inserts whose pinned relation is named like a CTE in scope.
pub(super) fn forget_shadowed(inserts: &mut [SqlConflictInsertFact], with: &With) {
    for insert in inserts {
        let shadowed = insert
            .source
            .pinned_relation
            .as_ref()
            .is_some_and(|pinned| {
                let name = pinned
                    .table
                    .rsplit('.')
                    .next()
                    .unwrap_or_default()
                    .trim_matches('"');
                with.cte_tables
                    .iter()
                    .any(|cte| cte.alias.name.value.eq_ignore_ascii_case(name))
            });
        if shadowed {
            insert.source.pinned_relation = None;
        }
    }
}
