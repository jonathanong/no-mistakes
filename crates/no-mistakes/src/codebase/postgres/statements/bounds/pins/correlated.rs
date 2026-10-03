use super::super::super::value::is_placeholder_ident;
use crate::codebase::postgres::idents::{ident_key, object_name_ident};
use sqlparser::ast::{Expr, Query, TableFactor, Visit, Visitor};
use std::collections::BTreeSet;
use std::ops::ControlFlow;

/// Whether `query` reads a column of a relation outside it, so the rows it returns depend on
/// the row being checked and its size says nothing about how many rows match.
///
/// `outer` holds the names the enclosing FROM items answer to. References are resolved one query
/// level at a time, innermost first: a qualified column is the level's own when one of its
/// relations has that name, and otherwise belongs to the level above. A bare column belongs to
/// the level above only when the level has no relation of its own to take it from; with one,
/// which of them owns it cannot be told without the catalog.
pub(super) fn reads_outer_rows(query: &Query, outer: &BTreeSet<String>) -> bool {
    let mut scan = Scan::default();
    let _ = query.visit(&mut scan);
    scan.bare_unresolved
        || scan
            .unresolved
            .iter()
            .any(|qualifier| outer.contains(qualifier))
}

/// What one query level mentions, until its relations are all known.
#[derive(Default)]
struct Frame {
    relations: BTreeSet<String>,
    qualifiers: Vec<String>,
    bare_column: bool,
}

#[derive(Default)]
struct Scan {
    stack: Vec<Frame>,
    /// Qualifiers no level of the query resolved, and whether a bare column reached the top.
    unresolved: Vec<String>,
    bare_unresolved: bool,
}

impl Visitor for Scan {
    type Break = ();

    fn pre_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.stack.push(Frame::default());
        ControlFlow::Continue(())
    }

    /// The relations of a level are complete only after it has been visited (the projection comes
    /// before FROM), so its references are resolved here and what remains moves up a level.
    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        let frame = self.stack.pop().unwrap_or_default();
        let up: Vec<String> = frame
            .qualifiers
            .into_iter()
            .filter(|qualifier| !frame.relations.contains(qualifier))
            .collect();
        let bare = frame.bare_column && frame.relations.is_empty();
        match self.stack.last_mut() {
            Some(parent) => {
                parent.qualifiers.extend(up);
                parent.bare_column |= bare;
            }
            None => {
                self.unresolved.extend(up);
                self.bare_unresolved |= bare;
            }
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
        let Some(frame) = self.stack.last_mut() else {
            return ControlFlow::Continue(());
        };
        match factor {
            TableFactor::Table { name, alias, .. } => {
                let own = alias
                    .as_ref()
                    .map(|alias| ident_key(&alias.name))
                    .or_else(|| object_name_ident(name).map(ident_key));
                frame.relations.extend(own);
            }
            TableFactor::Derived {
                alias: Some(alias), ..
            } => {
                frame.relations.insert(ident_key(&alias.name));
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        let Some(frame) = self.stack.last_mut() else {
            return ControlFlow::Continue(());
        };
        match expr {
            Expr::CompoundIdentifier(parts) if parts.len() >= 2 => {
                frame.qualifiers.push(ident_key(&parts[parts.len() - 2]));
            }
            Expr::Identifier(ident) if !is_placeholder_ident(&ident.value) => {
                frame.bare_column = true;
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}
