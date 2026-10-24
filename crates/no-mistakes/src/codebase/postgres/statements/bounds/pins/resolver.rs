mod canonical;
mod outputs;
mod sources;
use super::super::super::value::{is_placeholder_ident_at, PlaceholderPositions};
use super::correlated::{reads_outer_rows, Reads};
use crate::codebase::postgres::catalog::decoded_parts;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::statements::{
    SqlBareRead, SqlBoundItem, SqlBoundItemKind, SqlPinSource, SqlQualifiedRead,
};
use outputs::caller_outputs;
use sqlparser::ast::{Expr, Ident, Query};
use std::collections::{BTreeMap, BTreeSet};

/// Maps column references to FROM items by alias, or by table name when unaliased.
pub(in super::super) struct Resolver {
    names: Vec<(Option<String>, Option<String>)>,
    /// Only base relations take pins; a derived item is sized by its own query.
    tables: Vec<bool>,
    /// Independently caller-sized projected values, keyed by the item's visible column names.
    caller_outputs: Vec<BTreeSet<String>>,
    /// What each item answers to (its alias, else its table name): the qualifiers a subquery
    /// would use to read the row being checked.
    outer: BTreeSet<Vec<String>>,
    /// The CTE names in scope: a one-part table name that is one is not a base table.
    ctes: BTreeMap<String, Option<BTreeSet<String>>>,
    positions: Option<Vec<(u32, u32)>>,
}

/// What a pin's value is sized by, and the bare columns its subqueries read.
pub(super) struct Sourced {
    pub(super) source: SqlPinSource,
    pub(super) reads: Vec<SqlBareRead>,
    pub(super) qualified_reads: Vec<SqlQualifiedRead>,
}

impl Resolver {
    pub(in super::super) fn new(
        items: &[SqlBoundItem],
        ctes: BTreeMap<String, Option<BTreeSet<String>>>,
        positions: PlaceholderPositions<'_>,
    ) -> Self {
        let names: Vec<(Option<String>, Option<String>)> = items
            .iter()
            .map(|item| {
                let table = match &item.kind {
                    SqlBoundItemKind::Table(name) => decoded_parts(name).pop(),
                    _ => None,
                };
                (item.alias.clone(), table)
            })
            .collect();
        let tables = items
            .iter()
            .map(|item| matches!(item.kind, SqlBoundItemKind::Table(_)))
            .collect();
        let mut outer: BTreeSet<Vec<String>> = names
            .iter()
            .filter_map(|(alias, table)| alias.clone().or_else(|| table.clone()))
            .map(|name| vec![name])
            .collect();
        for item in items {
            // Even an alias matching the table's bare name hides its schema-qualified name.
            if !item.alias_explicit {
                if let SqlBoundItemKind::Table(name) = &item.kind {
                    outer.insert(decoded_parts(name));
                }
            }
        }
        Self {
            caller_outputs: items.iter().map(caller_outputs).collect(),
            names,
            tables,
            outer,
            ctes,
            positions: positions.map(<[(u32, u32)]>::to_vec),
        }
    }

    pub(in super::super) fn is_table(&self, item: usize) -> bool {
        self.tables[item]
    }

    /// What `query` reads of these items, so how far it depends on the row checked.
    pub(in super::super) fn reads(&self, query: &Query) -> Reads {
        reads_outer_rows(query, &self.outer, &self.ctes, self.positions.as_deref())
    }

    /// The item and column when `expr` is a bare column of one FROM item.
    pub(super) fn column(&self, expr: &Expr) -> Option<(usize, String)> {
        match unwrap_expr(expr) {
            // A recovered `${…}` interpolation is a bind, never a column.
            Expr::Identifier(ident)
                if is_placeholder_ident_at(ident, self.positions.as_deref()) =>
            {
                None
            }
            Expr::Identifier(ident) => (self.names.len() == 1).then(|| (0, ident_key(ident))),
            Expr::CompoundIdentifier(parts) => {
                let (qualifier, column) = qualified(parts)?;
                Some((self.item_named(&qualifier)?, column))
            }
            _ => None,
        }
    }

    fn item_named(&self, qualifier: &str) -> Option<usize> {
        let matches: Vec<usize> = self
            .names
            .iter()
            .enumerate()
            .filter(|(_, (alias, table))| match alias {
                Some(alias) => alias == qualifier,
                None => table.as_deref() == Some(qualifier),
            })
            .map(|(index, _)| index)
            .collect();
        (matches.len() == 1).then(|| matches[0])
    }
}

fn qualified(parts: &[Ident]) -> Option<(String, String)> {
    parts
        .iter()
        .rev()
        .nth(1)
        .zip(parts.last())
        .map(|(qualifier, column)| (ident_key(qualifier), ident_key(column)))
}

#[cfg(test)]
mod tests;
