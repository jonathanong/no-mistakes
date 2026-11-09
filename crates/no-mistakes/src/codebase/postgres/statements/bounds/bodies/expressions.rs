use super::super::super::value::PlaceholderPositions;
use super::super::{pins, query, Scope};
use crate::codebase::postgres::statements::SqlBoundItem;
use sqlparser::ast::{Expr, Query, TableFactor, Visit, Visitor};
use std::{cell::OnceCell, ops::ControlFlow};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Context {
    Other,
    Probe,
    Scalar,
    Pool(usize),
}

pub(super) fn collect<T: Visit>(
    expressions: &T,
    scope: &Scope,
    items: &[SqlBoundItem],
    positions: PlaceholderPositions<'_>,
) {
    let mut visitor = Expressions {
        scope,
        nested_scope: OnceCell::new(),
        items,
        positions,
        depth: 0,
        derived: 0,
        contexts: Vec::new(),
    };
    let _ = expressions.visit(&mut visitor);
}

struct Expressions<'a> {
    scope: &'a Scope,
    nested_scope: OnceCell<Scope>,
    items: &'a [SqlBoundItem],
    positions: PlaceholderPositions<'a>,
    depth: usize,
    derived: usize,
    contexts: Vec<Context>,
}

impl Visitor for Expressions<'_> {
    type Break = ();

    fn pre_visit_expr(&mut self, expression: &Expr) -> ControlFlow<()> {
        let context = match expression {
            Expr::Exists { .. } => Context::Probe,
            Expr::Subquery(query) => {
                let identity = query.as_ref() as *const Query as usize;
                if self.contexts.last() == Some(&Context::Pool(identity)) {
                    Context::Pool(identity)
                } else {
                    Context::Scalar
                }
            }
            Expr::InSubquery { subquery, .. } => {
                Context::Pool(subquery.as_ref() as *const Query as usize)
            }
            Expr::AnyOp { right, .. } | Expr::AllOp { right, .. } => {
                match crate::codebase::postgres::idents::unwrap_expr(right) {
                    Expr::Subquery(query) => Context::Pool(query.as_ref() as *const Query as usize),
                    _ => Context::Other,
                }
            }
            _ => Context::Other,
        };
        self.contexts.push(context);
        ControlFlow::Continue(())
    }

    fn post_visit_expr(&mut self, _: &Expr) -> ControlFlow<()> {
        self.contexts.pop();
        ControlFlow::Continue(())
    }

    fn pre_visit_query(&mut self, subquery: &Query) -> ControlFlow<()> {
        // FROM traversal owns derived projections; binding them again grows exponentially.
        if self.depth == 0 && self.derived == 0 {
            let scope = self
                .nested_scope
                .get_or_init(|| self.scope.with_scalar_scope(self.items));
            let bound = query::bound_query(subquery, scope, self.positions);
            let context = self.contexts.last().copied().unwrap_or(Context::Other);
            let reads = (context == Context::Scalar)
                .then(|| pins::reads_items(subquery, self.items, scope, self.positions));
            let probe = context == Context::Probe
                || reads
                    .as_ref()
                    .is_some_and(|reads| reads.certain || !reads.qualified.is_empty());
            if !probe {
                scope.record_subquery(
                    &bound,
                    subquery,
                    reads.map(|reads| scope.scalar_reads(reads.bare)),
                );
            }
        }
        self.depth += 1;
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.depth -= 1;
        ControlFlow::Continue(())
    }

    fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
        self.derived += usize::from(matches!(factor, TableFactor::Derived { .. }));
        ControlFlow::Continue(())
    }

    fn post_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
        self.derived -= usize::from(matches!(factor, TableFactor::Derived { .. }));
        ControlFlow::Continue(())
    }
}
