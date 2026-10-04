use super::super::super::value::PlaceholderPositions;
use super::super::functions::{function_kind_at, unnest_kind_at};
use super::super::using::Using;
use super::super::{pins, query, start, Scope};
use super::{alias_columns, alias_key, sql_name};
use crate::codebase::postgres::idents::{ident_key, object_name_ident, object_name_key};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind};
use sqlparser::ast::{Expr, JoinOperator, ObjectName, Spanned, TableFactor, TableWithJoins};

/// Gathers FROM items and the join conditions that restrict them, then resolves the pins.
pub(in crate::codebase::postgres::statements::bounds) struct Builder<'a> {
    scope: &'a Scope,
    positions: PlaceholderPositions<'a>,
    items: Vec<SqlBoundItem>,
    /// Join conditions, with the items each one restricts.
    conditions: Vec<(Vec<usize>, &'a Expr)>,
    usings: Vec<Using<'a>>,
    /// The next FROM item is the relation an `UPDATE` or `DELETE` changes.
    target_next: bool,
}

impl<'a> Builder<'a> {
    pub(in crate::codebase::postgres::statements::bounds) fn new(
        scope: &'a Scope,
        positions: PlaceholderPositions<'a>,
    ) -> Self {
        Self {
            scope,
            positions,
            items: Vec::new(),
            conditions: Vec::new(),
            usings: Vec::new(),
            target_next: false,
        }
    }

    /// Add the tables of an `UPDATE` or `DELETE`, whose first relation is the one it changes:
    /// a DML target is a physical relation even when a CTE shares its name.
    pub(in crate::codebase::postgres::statements::bounds) fn target_tables(
        &mut self,
        tables: &'a [TableWithJoins],
    ) {
        self.target_next = true;
        self.tables(tables);
    }

    pub(in crate::codebase::postgres::statements::bounds) fn tables(
        &mut self,
        tables: &'a [TableWithJoins],
    ) {
        for table in tables {
            let chain = self.items.len();
            self.factor(&table.relation);
            for join in &table.joins {
                let right = self.items.len();
                self.factor(&join.relation);
                let end = self.items.len();
                // ON restricts only the non-preserved side of an outer join.
                let restricted = match &join.join_operator {
                    JoinOperator::Join(_) | JoinOperator::Inner(_) => chain..end,
                    JoinOperator::Left(_) | JoinOperator::LeftOuter(_) => right..end,
                    JoinOperator::Right(_) | JoinOperator::RightOuter(_) => chain..right,
                    _ => continue,
                };
                if let Some(on) =
                    crate::codebase::postgres::statements::select::join_expr(&join.join_operator)
                {
                    self.conditions.push((restricted.collect(), on));
                } else if chain + 1 == right && right + 1 == end {
                    self.usings.extend(Using::of(
                        &join.join_operator,
                        restricted.collect(),
                        chain,
                        right,
                    ));
                }
            }
        }
    }

    pub(in crate::codebase::postgres::statements::bounds) fn finish(
        mut self,
        selection: Option<&Expr>,
    ) -> Vec<SqlBoundItem> {
        let resolver = pins::Resolver::new(&self.items, self.scope.names(), self.positions);
        let all: Vec<usize> = (0..self.items.len()).collect();
        let mut found = Vec::new();
        for (restricted, condition) in &self.conditions {
            pins::extract(
                condition,
                &resolver,
                restricted,
                self.scope,
                self.positions,
                &mut found,
            );
        }
        for using in &self.usings {
            using.pins(&resolver, self.positions, &mut found);
        }
        if let Some(selection) = selection {
            pins::extract(
                selection,
                &resolver,
                &all,
                self.scope,
                self.positions,
                &mut found,
            );
        }
        for (item, pin) in found {
            if !self.items[item].pins.contains(&pin) {
                self.items[item].pins.push(pin);
            }
        }
        self.items
    }

    fn factor(&mut self, factor: &'a TableFactor) {
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
                let column_aliases = alias_columns(alias);
                let alias = alias_key(alias).or_else(|| match &kind {
                    SqlBoundItemKind::Query(_) => Some(object_name_key(name)),
                    SqlBoundItemKind::Table(_) => object_name_ident(name).map(ident_key),
                    _ => None,
                });
                let mut item = SqlBoundItem::new(kind, alias, start(name.span()));
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
                item.column_aliases = alias_columns(alias);
                if *lateral {
                    let reads =
                        pins::reads_items(subquery, &self.items, self.scope, self.positions);
                    item.lateral = reads.certain;
                    item.lateral_reads = reads.bare;
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
        item.column_aliases = alias_columns(alias);
        self.items.push(item);
    }
}
