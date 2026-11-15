use super::super::value::PlaceholderPositions;
use crate::codebase::postgres::idents::ident_key;
use crate::codebase::postgres::statements::limit::FixedCount;
use sqlparser::ast::{Expr, TableFactor, TableWithJoins};
use std::collections::BTreeMap;

mod columns;
mod context;
mod examine;
mod outputs;
mod shape;
pub(in crate::codebase::postgres::statements::bounds) use columns::null_extended_sides;
use columns::{alias_names, relation_key};
pub(super) use context::RelationContext;
pub(in crate::codebase::postgres::statements::bounds) use examine::examine;
pub(in crate::codebase::postgres::statements::bounds) use outputs::of_query;
#[cfg(test)]
pub(super) mod work;
#[cfg(not(test))]
#[path = "limit_columns/work_disabled.rs"]
mod work;

#[derive(Clone)]
enum Columns {
    Any {
        table: Option<String>,
        null_extended: bool,
    },
    Named(BTreeMap<String, Option<FixedCount>>),
}

#[derive(Clone, Default)]
pub(super) struct Relations {
    names: BTreeMap<String, Columns>,
    /// Snapshot taken before this `FROM` list. A non-`LATERAL` derived table sees it,
    /// not same-level siblings added afterward.
    outer: BTreeMap<String, Columns>,
    outputs: BTreeMap<String, Option<BTreeMap<String, Option<FixedCount>>>>,
    opaque: bool,
    outer_opaque: bool,
}

impl Relations {
    pub(super) fn add(
        &mut self,
        factor: &TableFactor,
        positions: PlaceholderPositions<'_>,
    ) -> Option<String> {
        let columns = match factor {
            TableFactor::UNNEST {
                alias, array_exprs, ..
            } => columns::named_caller(
                alias,
                array_exprs.len(),
                array_exprs
                    .iter()
                    .all(|expr| columns::caller_input(expr, positions)),
            ),
            TableFactor::Function {
                alias, name, args, ..
            } => columns::named_caller(
                alias,
                args.len(),
                columns::function_caller(name, args, positions),
            ),
            TableFactor::Table { name, alias, .. } => {
                // A column alias list renames by position. The stored names are the
                // relation's own outputs, so a renamed reference is not that proof.
                if alias
                    .as_ref()
                    .is_some_and(|alias| !alias.columns.is_empty())
                {
                    Columns::Named(BTreeMap::new())
                } else {
                    columns::table_columns(self, name)
                }
            }
            TableFactor::Derived {
                lateral,
                alias,
                subquery,
                ..
            } => {
                let parent = self.proof_parent(*lateral);
                match of_query(subquery, positions, &parent, &alias_names(alias)) {
                    Some(map) => Columns::Named(map),
                    None => Columns::Any {
                        table: None,
                        null_extended: false,
                    },
                }
            }
            _ => {
                self.names.clear();
                self.opaque = true;
                return None;
            }
        };
        let key = relation_key(factor);
        if let Some(key) = &key {
            self.names.insert(key.clone(), columns);
        } else {
            self.opaque = true;
        }
        key
    }

    pub(super) fn record(
        &mut self,
        name: String,
        columns: Option<BTreeMap<String, Option<FixedCount>>>,
    ) {
        self.outputs.insert(name, columns);
    }

    pub(super) fn contains(&self, expr: &Expr) -> Option<FixedCount> {
        match expr {
            Expr::CompoundIdentifier(parts) if parts.len() == 2 => {
                self.lookup(&ident_key(&parts[0]), &ident_key(&parts[1]))
            }
            Expr::Identifier(id) if !self.opaque && self.names.len() == 1 => {
                match self.names.values().next() {
                    Some(Columns::Named(names)) => columns::named_kind(names, &ident_key(id)),
                    Some(Columns::Any { .. }) | None => None,
                }
            }
            _ => None,
        }
    }

    pub(super) fn catalog_column(&self, expr: &Expr) -> Option<(String, String)> {
        let Expr::CompoundIdentifier(parts) = expr else {
            return None;
        };
        if parts.len() != 2 {
            return None;
        }
        match self.names.get(&ident_key(&parts[0])) {
            Some(Columns::Any {
                table: Some(table),
                null_extended: false,
            }) => Some((table.clone(), ident_key(&parts[1]))),
            _ => None,
        }
    }

    pub(super) fn null_extend(&mut self, keys: &[String]) {
        for key in keys {
            match self.names.get_mut(key) {
                Some(Columns::Any { null_extended, .. }) => *null_extended = true,
                Some(Columns::Named(columns)) => {
                    for kind in columns.values_mut() {
                        *kind = None;
                    }
                }
                None => {}
            }
        }
    }

    pub(super) fn absorb(
        &mut self,
        tables: &[TableWithJoins],
        positions: PlaceholderPositions<'_>,
    ) {
        self.begin_from();
        for table in tables {
            let mut keys = Vec::new();
            if let Some(key) = self.add(&table.relation, positions) {
                keys.push(key);
            }
            for join in &table.joins {
                let (extend_left, extend_right) = columns::null_extended_sides(&join.join_operator);
                if extend_left {
                    self.null_extend(&keys);
                }
                let added = self.add(&join.relation, positions);
                if extend_right {
                    if let Some(key) = &added {
                        self.null_extend(std::slice::from_ref(key));
                    }
                }
                if let Some(key) = added {
                    keys.push(key);
                }
            }
        }
    }

    fn lookup(&self, relation: &str, column: &str) -> Option<FixedCount> {
        match self.names.get(relation) {
            Some(Columns::Any { .. }) => Some(FixedCount::Nullable),
            Some(Columns::Named(names)) => columns::named_kind(names, column),
            None => None,
        }
    }
}
