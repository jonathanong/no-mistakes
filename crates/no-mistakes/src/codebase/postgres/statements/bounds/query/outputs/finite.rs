//! A scalar cast cannot hide data-backed expansion or an opaque derived-column origin.
use super::super::super::super::value::{is_placeholder_ident_at, PlaceholderPositions};
use super::super::super::pins::Resolver;
use crate::codebase::postgres::{SqlBoundItem, SqlBoundItemKind};
use sqlparser::ast::{Expr, Query, Visit, Visitor};
use std::ops::ControlFlow;

pub(super) fn expression(
    expr: &Expr,
    items: &[SqlBoundItem],
    resolver: &Resolver,
    positions: PlaceholderPositions<'_>,
) -> bool {
    struct Finite<'a> {
        items: &'a [SqlBoundItem],
        resolver: &'a Resolver,
        positions: PlaceholderPositions<'a>,
    }
    impl Visitor for Finite<'_> {
        type Break = ();

        fn pre_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
            // Scalar-subquery row cardinality is distinct from proven source-value lineage.
            ControlFlow::Break(())
        }

        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            let sized = match expr {
                Expr::Function(function) => {
                    !super::super::super::functions::data_backed_projection(
                        function,
                        self.positions,
                    )
                }
                Expr::Identifier(ident) if is_placeholder_ident_at(ident, self.positions) => true,
                Expr::Identifier(_) | Expr::CompoundIdentifier(_) => self
                    .resolver
                    .column(expr)
                    .is_some_and(|source| column(self.items, &source)),
                _ => true,
            };
            if sized {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(())
            }
        }
    }
    // Full AST traversal includes casts, nested calls, argument/filter/window expressions,
    // without COALESCE/aggregate traversal boundaries that could hide an expanding child.
    expr.visit(&mut Finite {
        items,
        resolver,
        positions,
    })
    .is_continue()
}

fn column(items: &[SqlBoundItem], (index, name): &(usize, String)) -> bool {
    let Some(item) = items.get(*index) else {
        return false;
    };
    match &item.kind {
        SqlBoundItemKind::Table(_) | SqlBoundItemKind::Other => true,
        SqlBoundItemKind::Opaque => false,
        SqlBoundItemKind::Query(query) => {
            if query.has_opaque_row_inputs() {
                return false;
            }
            let mut outputs = query
                .outputs
                .iter()
                .enumerate()
                .filter(|(index, output)| {
                    item.column_aliases.get(*index).or(output.name.as_ref()) == Some(name)
                })
                .map(|(_, output)| output);
            let Some(output) = outputs.next() else {
                return false;
            };
            if outputs.next().is_some() {
                return false;
            }
            output.scalar
                || output.caller_sized
                || output
                    .source_column
                    .as_ref()
                    .is_some_and(|source| column(&query.items, source))
        }
    }
}
