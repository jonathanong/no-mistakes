use super::{pins, query, start, Scope};
use crate::codebase::postgres::idents::{ident_key, object_name_key};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind};
use sqlparser::ast::{
    Expr, JoinOperator, ObjectName, Select, Spanned, TableFactor, TableWithJoins,
};

pub(super) fn from_select(select: &Select, scope: &Scope) -> Vec<SqlBoundItem> {
    let mut builder = Builder::new(scope);
    builder.tables(&select.from);
    builder.finish(select.selection.as_ref())
}

pub(super) fn other((line, column): (usize, usize)) -> SqlBoundItem {
    SqlBoundItem {
        kind: SqlBoundItemKind::Other,
        alias: None,
        line,
        column,
        pins: Vec::new(),
    }
}

/// Gathers FROM items and the join conditions that restrict them, then resolves the pins.
pub(super) struct Builder<'a> {
    scope: &'a Scope,
    items: Vec<SqlBoundItem>,
    /// Join conditions, with the items each one restricts.
    conditions: Vec<(Vec<usize>, &'a Expr)>,
}

impl<'a> Builder<'a> {
    pub(super) fn new(scope: &'a Scope) -> Self {
        Self {
            scope,
            items: Vec::new(),
            conditions: Vec::new(),
        }
    }

    pub(super) fn tables(&mut self, tables: &'a [TableWithJoins]) {
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
                }
            }
        }
    }

    pub(super) fn finish(mut self, selection: Option<&Expr>) -> Vec<SqlBoundItem> {
        let resolver = pins::Resolver::new(&self.items);
        let all: Vec<usize> = (0..self.items.len()).collect();
        let mut found = Vec::new();
        for (restricted, condition) in &self.conditions {
            pins::extract(condition, &resolver, restricted, self.scope, &mut found);
        }
        if let Some(selection) = selection {
            pins::extract(selection, &resolver, &all, self.scope, &mut found);
        }
        for (item, pin) in found {
            if !self.items[item].pins.contains(&pin) {
                self.items[item].pins.push(pin);
            }
        }
        self.items
    }

    fn factor(&mut self, factor: &'a TableFactor) {
        match factor {
            TableFactor::Table {
                name, alias, args, ..
            } => {
                let kind = if args.is_some() {
                    SqlBoundItemKind::Other
                } else {
                    self.named(name)
                };
                // An unaliased CTE reference is addressed by the CTE's name.
                let alias = alias_key(alias).or_else(|| {
                    matches!(kind, SqlBoundItemKind::Query(_)).then(|| object_name_key(name))
                });
                self.push(kind, alias, start(name.span()));
            }
            TableFactor::Derived {
                subquery, alias, ..
            } => {
                let bound = query::bound_query(subquery, self.scope);
                let kind = SqlBoundItemKind::Query(bound);
                self.push(kind, alias_key(alias), start(subquery.span()));
            }
            TableFactor::NestedJoin {
                table_with_joins, ..
            } => self.tables(std::slice::from_ref(&**table_with_joins)),
            other => self.push(SqlBoundItemKind::Other, None, start(other.span())),
        }
    }

    /// A CTE reference carries the CTE's bound; anything else is a base relation.
    fn named(&self, name: &ObjectName) -> SqlBoundItemKind {
        let key = object_name_key(name);
        match self.scope.get(&key).filter(|_| !key.contains('.')) {
            Some(bound) => SqlBoundItemKind::Query(bound.clone()),
            None => SqlBoundItemKind::Table(key),
        }
    }

    fn push(
        &mut self,
        kind: SqlBoundItemKind,
        alias: Option<String>,
        (line, column): (usize, usize),
    ) {
        self.items.push(SqlBoundItem {
            kind,
            alias,
            line,
            column,
            pins: Vec::new(),
        });
    }
}

fn alias_key(alias: &Option<sqlparser::ast::TableAlias>) -> Option<String> {
    alias.as_ref().map(|alias| ident_key(&alias.name))
}
