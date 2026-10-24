use super::super::super::{
    functions::{function_kind_at, unnest_kind_at},
    pins, query, start,
};
use super::super::{alias_columns, alias_key, sql_name};
use super::Builder;
use crate::codebase::postgres::idents::{ident_key, object_name_ident, object_name_key};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind};
use sqlparser::ast::{ObjectName, Spanned, TableFactor};

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
            } => self.push(
                unnest_kind_at(array_exprs, self.positions),
                alias,
                start(factor.span()),
            ),
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
