use super::super::super::value::PlaceholderPositions;
use super::super::{pins, query, Scope};
use crate::codebase::postgres::statements::SqlBoundItem;
use sqlparser::ast::{Expr, Query, TableFactor, Visit, Visitor};
use std::ops::ControlFlow;

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
            let bound = query::bound_query(subquery, self.scope, self.positions);
            let context = self.contexts.last().copied().unwrap_or(Context::Other);
            let probe = context == Context::Probe
                || (context == Context::Scalar && self.correlated(subquery));
            if !probe {
                self.scope.record_subquery(&bound, subquery);
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

impl Expressions<'_> {
    fn correlated(&self, subquery: &Query) -> bool {
        let reads = pins::reads_items(subquery, self.items, self.scope, self.positions);
        reads.certain || !reads.qualified.is_empty()
    }
}
