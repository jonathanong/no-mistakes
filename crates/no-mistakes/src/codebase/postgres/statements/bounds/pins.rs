use super::{query, Scope};
use crate::codebase::postgres::idents::{ident_key, unwrap_expr, visit_child_exprs};
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundItemKind, SqlBoundPin, SqlPinSource,
};
use sqlparser::ast::{BinaryOperator, Expr, Ident};
use std::collections::BTreeSet;

/// Maps column references to FROM items by alias, or by table name when unaliased.
pub(super) struct Resolver {
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
    pub(super) fn new(items: &[SqlBoundItem]) -> Self {
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

    /// The item and column when `expr` is a bare column of one FROM item.
    fn column(&self, expr: &Expr) -> Option<(usize, String)> {
        match unwrap_expr(expr) {
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
            expr @ (Expr::Identifier(_) | Expr::CompoundIdentifier(_)) => match self.column(expr) {
                Some((item, _)) => {
                    found.items.insert(item);
                }
                None => found.unknown = true,
            },
            other => visit_child_exprs(other, &mut |child| self.refs(child, found)),
        }
    }

    fn source(&self, value: &Expr, pinned: usize) -> Option<SqlPinSource> {
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

/// Collect the pins that the conjuncts of `expr` impose on the `restricted` items.
pub(super) fn extract(
    expr: &Expr,
    resolver: &Resolver,
    restricted: &[usize],
    scope: &Scope,
    out: &mut Vec<(usize, SqlBoundPin)>,
) {
    let mut pin = |column: &Expr, source: Option<SqlPinSource>| {
        if let (Some((item, column)), Some(source)) = (resolver.column(column), source) {
            if restricted.contains(&item) && resolver.tables[item] {
                out.push((item, SqlBoundPin { column, source }));
            }
        }
    };
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            extract(left, resolver, restricted, scope, out);
            extract(right, resolver, restricted, scope, out);
        }
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        }
        | Expr::IsNotDistinctFrom(left, right) => {
            if let Some((item, _)) = resolver.column(left) {
                pin(left, resolver.source(right, item));
            }
            if let Some((item, _)) = resolver.column(right) {
                pin(right, resolver.source(left, item));
            }
        }
        Expr::InList {
            expr,
            list,
            negated: false,
        } => {
            if let Some((item, _)) = resolver.column(expr) {
                let sources: Option<Vec<SqlPinSource>> = list
                    .iter()
                    .map(|value| resolver.source(value, item))
                    .collect();
                pin(expr, sources.map(merge));
            }
        }
        Expr::InSubquery {
            expr,
            subquery,
            negated: false,
        } => pin(
            expr,
            Some(SqlPinSource::Query(query::bound_query(subquery, scope))),
        ),
        Expr::AnyOp {
            left,
            compare_op: BinaryOperator::Eq,
            right,
            ..
        } => {
            if let Some((item, _)) = resolver.column(left) {
                let source = match unwrap_expr(right) {
                    Expr::Subquery(subquery) => {
                        Some(SqlPinSource::Query(query::bound_query(subquery, scope)))
                    }
                    other => resolver.source(other, item),
                };
                pin(left, source);
            }
        }
        _ => {}
    }
}

/// An `IN` list is sized by the caller, plus whatever other items its elements name.
fn merge(sources: Vec<SqlPinSource>) -> SqlPinSource {
    let items: BTreeSet<usize> = sources
        .into_iter()
        .flat_map(|source| match source {
            SqlPinSource::Items(items) => items,
            _ => Vec::new(),
        })
        .collect();
    if items.is_empty() {
        SqlPinSource::Value
    } else {
        SqlPinSource::Items(items.into_iter().collect())
    }
}
