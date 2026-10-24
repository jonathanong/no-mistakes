mod factors;
use super::super::super::value::PlaceholderPositions;
use super::super::using::Using;
use super::super::{pins, Scope};
use crate::codebase::postgres::statements::SqlBoundItem;
use sqlparser::ast::{Expr, JoinOperator, TableWithJoins};

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
        if self.conditions.is_empty() && self.usings.is_empty() && selection.is_none() {
            return self.items;
        }
        let resolver = pins::Resolver::new(&self.items, self.scope.names(), self.positions);
        let all: Vec<usize> = (0..self.items.len()).collect();
        let mut found = Vec::new();
        let mut conjuncts = Vec::new();
        let mut nonnull = Vec::new();
        for (restricted, condition) in &self.conditions {
            pins::extract(
                condition,
                &resolver,
                restricted,
                self.scope,
                self.positions,
                &mut found,
            );
            pins::collect_evidence(
                condition,
                &resolver,
                restricted,
                &mut conjuncts,
                &mut nonnull,
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
            pins::collect_evidence(selection, &resolver, &all, &mut conjuncts, &mut nonnull);
        }
        for (item, pin) in found {
            if !self.items[item].pins.contains(&pin) {
                self.items[item].pins.push(pin);
            }
        }
        for (item, conjunct) in conjuncts {
            if !self.items[item].predicate_conjuncts.contains(&conjunct) {
                self.items[item].predicate_conjuncts.push(conjunct);
            }
        }
        for (item, column) in nonnull {
            if !self.items[item].nonnull_columns.contains(&column) {
                self.items[item].nonnull_columns.push(column);
            }
        }
        if let Some(selection) = selection {
            super::alternatives::collect(
                selection,
                &resolver,
                self.scope,
                self.positions,
                &mut self.items,
            );
        }
        self.items
    }
}

mod factors;
