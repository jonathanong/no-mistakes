use super::super::super::value::is_placeholder_ident;
use super::correlated::{reads_outer_rows, Reads};
use crate::codebase::postgres::idents::{
    ident_key, object_name_ident, unwrap_expr, visit_child_exprs,
};
use crate::codebase::postgres::statements::{
    SqlBareRead, SqlBoundItem, SqlBoundItemKind, SqlPinSource,
};
use sqlparser::ast::{Expr, FunctionArguments, Ident, ObjectName, Query};
use std::collections::{BTreeMap, BTreeSet};

/// Built-in functions that return a different value for each row they are evaluated for, so an
/// equality against one picks out no fixed row. A function that is not listed (a user-defined
/// one included) is assumed to be row-invariant: its volatility is not part of the facts.
#[rustfmt::skip]
const ROW_VARIANT: &[&str] = &[
    "nextval", "currval", "lastval", "setval", "random", "random_normal", "setseed",
    "gen_random_uuid", "uuidv4", "uuidv7", "uuid_generate_v1", "uuid_generate_v1mc",
    "uuid_generate_v4", "clock_timestamp", "timeofday", "pg_sleep",
];

/// Maps column references to FROM items by alias, or by table name when unaliased.
pub(in super::super) struct Resolver {
    names: Vec<(Option<String>, Option<String>)>,
    /// Only base relations take pins; a derived item is sized by its own query.
    tables: Vec<bool>,
    /// What each item answers to (its alias, else its table name): the qualifiers a subquery
    /// would use to read the row being checked.
    outer: BTreeSet<String>,
    /// The CTE names in scope: a one-part table name that is one is not a base table.
    ctes: BTreeMap<String, Option<BTreeSet<String>>>,
}

/// What an expression refers to among the FROM items.
#[derive(Default)]
struct Refs {
    items: BTreeSet<usize>,
    /// A column the FROM items cannot explain: an outer reference or an ambiguous bare name.
    unknown: bool,
    /// Bare columns that subqueries of the expression read, if no table of theirs has them.
    reads: Vec<SqlBareRead>,
}

/// What a pin's value is sized by, and the bare columns its subqueries read.
pub(super) struct Sourced {
    pub(super) source: SqlPinSource,
    pub(super) reads: Vec<SqlBareRead>,
}

impl Resolver {
    pub(in super::super) fn new(
        items: &[SqlBoundItem],
        ctes: BTreeMap<String, Option<BTreeSet<String>>>,
    ) -> Self {
        let names: Vec<(Option<String>, Option<String>)> = items
            .iter()
            .map(|item| {
                let table = match &item.kind {
                    SqlBoundItemKind::Table(name) => name.rsplit('.').next().map(str::to_string),
                    _ => None,
                };
                (item.alias.clone(), table)
            })
            .collect();
        let tables = items
            .iter()
            .map(|item| matches!(item.kind, SqlBoundItemKind::Table(_)))
            .collect();
        let outer = names
            .iter()
            .filter_map(|(alias, table)| alias.clone().or_else(|| table.clone()))
            .collect();
        Self {
            names,
            tables,
            outer,
            ctes,
        }
    }

    pub(in super::super) fn is_table(&self, item: usize) -> bool {
        self.tables[item]
    }

    /// What `query` reads of these items, so how far it depends on the row checked.
    pub(in super::super) fn reads(&self, query: &Query) -> Reads {
        reads_outer_rows(query, &self.outer, &self.ctes)
    }

    /// The item and column when `expr` is a bare column of one FROM item.
    pub(super) fn column(&self, expr: &Expr) -> Option<(usize, String)> {
        match unwrap_expr(expr) {
            // A recovered `${…}` interpolation is a bind, never a column.
            Expr::Identifier(ident) if is_placeholder_ident(&ident.value) => None,
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

    fn refs(&self, expr: &Expr, found: &mut Refs) {
        match unwrap_expr(expr) {
            // A scalar subquery is a value the statement does not size, unless it reads the row.
            Expr::Subquery(query) => {
                let reads = self.reads(query);
                found.unknown |= reads.certain;
                found.reads.extend(reads.bare);
            }
            Expr::Exists { .. } => {}
            // An explicit collation changes what `=` matches, whatever the key's own collation
            // (`email = $1 COLLATE "case_insensitive"`): the value fixes no row.
            Expr::Collate { .. } => found.unknown = true,
            // A function that differs per row is not a fixed value, and `ARRAY(SELECT …)` is the
            // rows of a query, not a value the caller sized.
            Expr::Function(function)
                if is_row_variant(&function.name)
                    || matches!(function.args, FunctionArguments::Subquery(_)) =>
            {
                found.unknown = true
            }
            // A bind, whether `$1` or an interpolation recovered from a template literal.
            Expr::Identifier(ident) if is_placeholder_ident(&ident.value) => {}
            expr @ (Expr::Identifier(_) | Expr::CompoundIdentifier(_)) => match self.column(expr) {
                Some((item, _)) => {
                    found.items.insert(item);
                }
                None => found.unknown = true,
            },
            other => visit_child_exprs(other, &mut |child| self.refs(child, found)),
        }
    }

    pub(super) fn source(&self, value: &Expr, pinned: usize) -> Option<Sourced> {
        let mut found = Refs::default();
        self.refs(value, &mut found);
        if found.unknown || found.items.contains(&pinned) {
            return None;
        }
        let source = if found.items.is_empty() {
            SqlPinSource::Value
        } else {
            SqlPinSource::Items(found.items.into_iter().collect())
        };
        Some(Sourced {
            source,
            reads: found.reads,
        })
    }
}

fn is_row_variant(name: &ObjectName) -> bool {
    object_name_ident(name).is_some_and(|ident| ROW_VARIANT.contains(&ident_key(ident).as_str()))
}

fn qualified(parts: &[Ident]) -> Option<(String, String)> {
    let column = ident_key(parts.last()?);
    let qualifier = ident_key(parts.get(parts.len().checked_sub(2)?)?);
    Some((qualifier, column))
}
