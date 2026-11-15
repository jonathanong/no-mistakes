//! Plain GROUP BY proofs: one value per key, or a key that comes from a bounded input.
use super::super::pins::Resolver;
use super::super::Scope;
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::value::{is_placeholder_ident_at, PlaceholderPositions};
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundItemKind, SqlBoundOutput, SqlBoundPin, SqlBoundQuery, SqlGroupKey,
};
use sqlparser::ast::{Expr, GroupByExpr, Query, Select, Value};

pub(super) struct Proof {
    pub(super) constant: bool,
    pub(super) keys: Vec<SqlGroupKey>,
}

pub(super) fn prove(
    select: &Select,
    items: &[SqlBoundItem],
    scope: &Scope,
    positions: PlaceholderPositions<'_>,
) -> Proof {
    let empty = Proof {
        constant: false,
        keys: Vec::new(),
    };
    let GroupByExpr::Expressions(expressions, modifiers) = &select.group_by else {
        return empty;
    };
    if expressions.is_empty()
        || !modifiers.is_empty()
        || super::super::aggregate::projection_may_erase_input(select, scope)
        || super::super::aggregate::distinct_on_expands(select)
    {
        return empty;
    }
    let resolver = Resolver::new(items, scope.names(), positions);
    let mut keys = Vec::new();
    for expr in expressions {
        let Some(key) = key(expr, items, &resolver, positions) else {
            return empty;
        };
        keys.push(key);
    }
    if keys.iter().all(|key| key.constant) {
        Proof {
            constant: true,
            keys: Vec::new(),
        }
    } else {
        Proof {
            constant: false,
            keys,
        }
    }
}

/// A set-returning ORDER BY multiplies grouped rows, so that order drops only the group proof.
pub(super) fn apply_order(bound: &mut SqlBoundQuery, query: &Query, scope: &Scope) {
    if super::super::aggregate::order_may_erase_input(query, scope) {
        bound.group_constant = false;
        bound.group_keys.clear();
        return;
    }
    if bound.group_constant {
        bound.capped = true;
    }
}

fn key(
    expr: &Expr,
    items: &[SqlBoundItem],
    resolver: &Resolver,
    positions: PlaceholderPositions<'_>,
) -> Option<SqlGroupKey> {
    if let Some((item, column)) = resolver.column(expr) {
        return Some(SqlGroupKey {
            constant: column_single(items, item, &column, 0),
            item: Some(item),
        });
    }
    literal_constant(expr, positions).then_some(SqlGroupKey {
        item: None,
        constant: true,
    })
}

/// An integer literal in GROUP BY is an output ordinal, not a one-row constant.
fn literal_constant(expr: &Expr, positions: PlaceholderPositions<'_>) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => !matches!(value.value, Value::Number(..)),
        Expr::Identifier(ident) => is_placeholder_ident_at(ident, positions),
        _ => false,
    }
}

fn column_single(items: &[SqlBoundItem], item: usize, column: &str, depth: usize) -> bool {
    if depth > 8 {
        return false;
    }
    let current = &items[item];
    if pin_single(&current.pins, column) {
        return true;
    }
    let SqlBoundItemKind::Query(query) = &current.kind else {
        return false;
    };
    let matched: Vec<_> = query
        .outputs
        .iter()
        .enumerate()
        .filter(|(index, output)| {
            current
                .column_aliases
                .get(*index)
                .map(String::as_str)
                .or(output.name.as_deref())
                == Some(column)
        })
        .map(|(_, output)| output)
        .collect();
    !matched.is_empty()
        && matched
            .iter()
            .all(|output| output_single(query, output, depth))
}

fn output_single(query: &SqlBoundQuery, output: &SqlBoundOutput, depth: usize) -> bool {
    if output.constant_null || output.caller_sized {
        return true;
    }
    let Some((item, column)) = &output.source_column else {
        return false;
    };
    column_single(&query.items, *item, column, depth + 1)
}

fn pin_single(pins: &[SqlBoundPin], column: &str) -> bool {
    pins.iter().any(|pin| {
        pin.single_value
            && pin.alternative.is_none()
            && pin.column == column
            && pin.expression.is_none()
            && !pin.null_safe
            && !pin.unqualified_candidate
            && pin.reads.is_empty()
            && pin.qualified_reads.is_empty()
    })
}
