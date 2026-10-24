//! A scalar LIMIT needs a guaranteed caller-owned count, not merely a bounded source row.
use super::super::value::PlaceholderPositions;
use super::Scope;
use crate::codebase::postgres::idents::{ident_key, object_name_key, unwrap_expr};
use sqlparser::ast::{
    Expr, LimitClause, Query, Select, SelectItem, SetExpr, TableAliasColumnDef, TableFactor,
};
use std::collections::BTreeSet;

pub(super) fn is_limited(
    query: &Query,
    scope: &Scope,
    positions: PlaceholderPositions<'_>,
) -> bool {
    let count = match &query.limit_clause {
        Some(LimitClause::LimitOffset {
            limit: Some(expr), ..
        })
        | Some(LimitClause::OffsetCommaLimit { limit: expr, .. }) => expr,
        _ => return false,
    };
    let Expr::Subquery(query) = unwrap_expr(count) else {
        return false;
    };
    single_output(query, scope, positions).is_some_and(|outputs| outputs.len() == 1 && outputs[0].1)
}

pub(super) fn outputs(
    query: &Query,
    scope: &Scope,
    positions: PlaceholderPositions<'_>,
    aliases: &[TableAliasColumnDef],
) -> Option<BTreeSet<String>> {
    Some(
        single_output(query, scope, positions)?
            .into_iter()
            .enumerate()
            .filter_map(|(index, (name, fixed))| {
                fixed
                    .then(|| aliases.get(index).map(|a| ident_key(&a.name)).or(name))
                    .flatten()
            })
            .collect(),
    )
}

fn single_output(
    query: &Query,
    scope: &Scope,
    positions: PlaceholderPositions<'_>,
) -> Option<Vec<(Option<String>, bool)>> {
    // Any row-removing construct could turn the scalar result into NULL (LIMIT ALL).
    if query.with.is_some()
        || query.limit_clause.is_some()
        || query.fetch.is_some()
        || query.order_by.is_some()
    {
        return None;
    }
    let SetExpr::Select(select) = &*query.body else {
        return None;
    };
    let source = source(select, scope)?;
    select
        .projection
        .iter()
        .map(|item| {
            let (expr, name) = match item {
                SelectItem::ExprWithAlias { expr, alias } => (expr, Some(ident_key(alias))),
                SelectItem::UnnamedExpr(expr) => (
                    expr,
                    match expr {
                        Expr::Identifier(id) => Some(ident_key(id)),
                        _ => None,
                    },
                ),
                _ => return None,
            };
            let fixed = super::super::limit::is_fixed_at(expr, positions)
                || match unwrap_expr(expr) {
                    Expr::Identifier(id) => source
                        .as_ref()
                        .is_some_and(|(_, counts)| counts.contains(&ident_key(id))),
                    Expr::CompoundIdentifier(parts) if parts.len() == 2 => {
                        source.as_ref().is_some_and(|(alias, counts)| {
                            *alias == ident_key(&parts[0]) && counts.contains(&ident_key(&parts[1]))
                        })
                    }
                    _ => false,
                };
            fixed.then_some((name, fixed))
        })
        .collect()
}

fn source<'a>(select: &Select, scope: &'a Scope) -> Option<Option<(String, &'a BTreeSet<String>)>> {
    if select.selection.is_some()
        || select.having.is_some()
        || select.qualify.is_some()
        || !matches!(&select.group_by, sqlparser::ast::GroupByExpr::Expressions(expressions, _) if expressions.is_empty())
    {
        return None;
    }
    match select.from.as_slice() {
        [] => Some(None),
        [table] if table.joins.is_empty() => {
            let TableFactor::Table {
                name,
                alias,
                args: None,
                ..
            } = &table.relation
            else {
                return None;
            };
            if name.0.len() != 1
                || alias
                    .as_ref()
                    .is_some_and(|alias| !alias.columns.is_empty())
            {
                return None;
            }
            let key = object_name_key(name);
            let counts = scope.fixed_counts(&key)?;
            // Empty still records one guaranteed row, but no usable LIMIT values.
            let qualifier = alias.as_ref().map_or(key, |alias| ident_key(&alias.name));
            Some(Some((qualifier, counts)))
        }
        _ => None,
    }
}
