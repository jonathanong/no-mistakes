use super::super::super::value::is_placeholder_ident;
use crate::codebase::postgres::idents::{ident_key, object_name_ident};
use sqlparser::ast::{Expr, Query, TableFactor, Visit, Visitor};
use std::collections::BTreeSet;
use std::ops::ControlFlow;

/// Whether `query` reads a column of a relation outside it, so the rows it returns depend on
/// the row being checked and its size says nothing about how many rows match.
///
/// `outer` holds the names the enclosing FROM items answer to. A qualified column is outer when
/// its qualifier is one of them and no relation of the query shadows it. A bare column is outer
/// only when the query has no relation of its own to take it from; with one, which of them owns
/// it cannot be told without the catalog.
pub(super) fn reads_outer_rows(query: &Query, outer: &BTreeSet<String>) -> bool {
    let mut scan = Scan::default();
    let _ = query.visit(&mut scan);
    let qualified = scan
        .qualifiers
        .iter()
        .any(|qualifier| outer.contains(qualifier) && !scan.relations.contains(qualifier));
    qualified || (scan.relations.is_empty() && scan.bare_column)
}

#[derive(Default)]
struct Scan {
    /// The names the query's own relations answer to, at any depth.
    relations: BTreeSet<String>,
    qualifiers: Vec<String>,
    bare_column: bool,
}

impl Visitor for Scan {
    type Break = ();

    fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
        match factor {
            TableFactor::Table { name, alias, .. } => {
                let own = alias
                    .as_ref()
                    .map(|alias| ident_key(&alias.name))
                    .or_else(|| object_name_ident(name).map(ident_key));
                self.relations.extend(own);
            }
            TableFactor::Derived {
                alias: Some(alias), ..
            } => {
                self.relations.insert(ident_key(&alias.name));
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        match expr {
            Expr::CompoundIdentifier(parts) if parts.len() >= 2 => {
                self.qualifiers.push(ident_key(&parts[parts.len() - 2]));
            }
            Expr::Identifier(ident) if !is_placeholder_ident(&ident.value) => {
                self.bare_column = true;
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}
