use super::super::super::value::is_placeholder_ident;
use crate::codebase::postgres::idents::{ident_key, unwrap_expr, visit_child_exprs};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlPinSource};
use sqlparser::ast::{Expr, Ident};
use std::collections::BTreeSet;

/// Maps column references to FROM items by alias, or by table name when unaliased.
pub(in super::super) struct Resolver {
    names: Vec<(Option<String>, Option<String>)>,
    /// Only base relations take pins; a derived item is sized by its own query.
    tables: Vec<bool>,
}

/// What an expression refers to among the FROM items.
#[derive(Default)]
struct Refs {
    items: BTreeSet<usize>,
    /// A column the FROM items cannot explain: an outer reference or an ambiguous bare name.
    unknown: bool,
}

impl Resolver {
    pub(in super::super) fn new(items: &[SqlBoundItem]) -> Self {
        let names = items
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
        Self { names, tables }
    }

    pub(super) fn is_table(&self, item: usize) -> bool {
        self.tables[item]
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
            // A scalar subquery is a value the statement does not size.
            Expr::Subquery(_) | Expr::Exists { .. } => {}
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

    pub(super) fn source(&self, value: &Expr, pinned: usize) -> Option<SqlPinSource> {
        let mut found = Refs::default();
        self.refs(value, &mut found);
        if found.unknown || found.items.contains(&pinned) {
            return None;
        }
        Some(if found.items.is_empty() {
            SqlPinSource::Value
        } else {
            SqlPinSource::Items(found.items.into_iter().collect())
        })
    }
}

fn qualified(parts: &[Ident]) -> Option<(String, String)> {
    let column = ident_key(parts.last()?);
    let qualifier = ident_key(parts.get(parts.len().checked_sub(2)?)?);
    Some((qualifier, column))
}
