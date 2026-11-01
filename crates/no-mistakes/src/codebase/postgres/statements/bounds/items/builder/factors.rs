use super::super::super::{
    functions::{function_kind_at, unnest_kind_at},
    pins, query, start,
};
use super::super::{alias_columns, alias_key, sql_name};
use super::Builder;
use crate::codebase::postgres::idents::{
    ident_key, object_name_ident, object_name_key, unwrap_expr,
};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind};
use sqlparser::ast::{Expr, FunctionArg, FunctionArgExpr, ObjectName, Spanned, TableFactor};

impl<'a> Builder<'a> {
    pub(super) fn factor(&mut self, factor: &'a TableFactor) {
        let target = std::mem::take(&mut self.target_next);
        match factor {
            TableFactor::Table {
                name, alias, args, ..
            } => {
                let kind = if let Some(args) = args {
                    function_kind_at(name, args, self.positions)
                } else if target {
                    SqlBoundItemKind::Table(sql_name(name))
                } else {
                    self.named(name)
                };
                // An unaliased relation is addressed by its own name: a CTE's, or a table's
                // bare name (`orders.id` for `public.orders`).
                let alias_explicit = alias.is_some();
                let column_aliases = alias_columns(alias);
                let alias = alias_key(alias).or_else(|| match &kind {
                    SqlBoundItemKind::Query(_) => Some(object_name_key(name)),
                    SqlBoundItemKind::Table(_) => object_name_ident(name).map(ident_key),
                    _ => None,
                });
                let mut item = SqlBoundItem::new(kind, alias, start(name.span()));
                item.alias_explicit = alias_explicit;
                item.column_aliases = column_aliases;
                self.items.push(item);
            }
            TableFactor::Derived {
                lateral,
                subquery,
                alias,
                ..
            } => {
                let bound = query::bound_query(subquery, self.scope, self.positions);
                let mut item = SqlBoundItem::new(
                    SqlBoundItemKind::Query(bound),
                    alias_key(alias),
                    start(subquery.span()),
                );
                item.alias_explicit = alias.is_some();
                item.column_aliases = alias_columns(alias);
                if *lateral {
                    let reads =
                        pins::reads_items(subquery, &self.items, self.scope, self.positions);
                    item.lateral = reads.certain;
                    item.lateral_reads = reads.bare;
                    item.lateral_qualified_reads = reads.qualified;
                }
                self.items.push(item);
            }
            TableFactor::NestedJoin {
                table_with_joins, ..
            } => self.tables(std::slice::from_ref(&**table_with_joins)),
            // `unnest(…)` is its own table factor: sized by the arrays it is given.
            TableFactor::UNNEST {
                array_exprs, alias, ..
            } => {
                let kind = if array_exprs
                    .iter()
                    .all(|expr| caller_column(expr, self.scope))
                {
                    SqlBoundItemKind::Other
                } else {
                    unnest_kind_at(array_exprs, self.positions)
                };
                self.push(kind, alias, start(factor.span()));
            }
            // `LATERAL unnest(...)` is a function table factor, not `UNNEST`.
            TableFactor::Function {
                name, args, alias, ..
            } => {
                let kind = lateral_unnest_kind(name, args, self.scope, self.positions);
                if matches!(kind, SqlBoundItemKind::Other) {
                    self.push(kind, alias, start(factor.span()));
                } else {
                    self.push(SqlBoundItemKind::Opaque, &None, start(factor.span()));
                }
            }
            other => self.push(SqlBoundItemKind::Opaque, &None, start(other.span())),
        }
    }

    /// A CTE reference carries the CTE's bound; anything else is a base relation.
    fn named(&self, name: &ObjectName) -> SqlBoundItemKind {
        // A one-part name is a CTE reference when a CTE has that name.
        let cte = self
            .scope
            .get(&object_name_key(name))
            .filter(|_| name.0.len() == 1);
        match cte {
            Some(bound) => SqlBoundItemKind::Query(bound.clone()),
            None => SqlBoundItemKind::Table(sql_name(name)),
        }
    }

    fn push(
        &mut self,
        kind: SqlBoundItemKind,
        alias: &Option<sqlparser::ast::TableAlias>,
        at: (usize, usize),
    ) {
        let mut item = SqlBoundItem::new(kind, alias_key(alias), at);
        item.alias_explicit = alias.is_some();
        item.column_aliases = alias_columns(alias);
        self.items.push(item);
    }
}

fn lateral_unnest_kind(
    name: &ObjectName,
    args: &[FunctionArg],
    scope: &super::super::super::Scope,
    positions: super::super::super::super::value::PlaceholderPositions<'_>,
) -> SqlBoundItemKind {
    if object_name_ident(name).is_none_or(|ident| ident_key(ident) != "unnest") {
        return SqlBoundItemKind::Opaque;
    }
    let Some(exprs) = arg_exprs(args) else {
        return SqlBoundItemKind::Opaque;
    };
    if exprs.iter().all(|expr| caller_column(expr, scope)) {
        return SqlBoundItemKind::Other;
    }
    unnest_kind_at(&exprs, positions)
}

fn arg_exprs(args: &[FunctionArg]) -> Option<Vec<Expr>> {
    args.iter()
        .map(|arg| match arg {
            FunctionArg::Unnamed(FunctionArgExpr::Expr(expr))
            | FunctionArg::Named {
                arg: FunctionArgExpr::Expr(expr),
                ..
            } => Some(expr.clone()),
            _ => None,
        })
        .collect()
}

/// `unnest(cte.column)` is caller-sized when that returned column is a caller value.
fn caller_column(expr: &Expr, scope: &super::super::super::Scope) -> bool {
    let Expr::CompoundIdentifier(parts) = unwrap_expr(expr) else {
        return false;
    };
    let [relation, column] = parts.as_slice() else {
        return false;
    };
    let column = ident_key(column);
    scope.get(&ident_key(relation)).is_some_and(|bound| {
        bound
            .outputs
            .iter()
            .any(|output| output.caller_sized && output.name.as_deref() == Some(column.as_str()))
    })
}
