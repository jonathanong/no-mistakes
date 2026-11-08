use super::{Scope, SqlBoundFact, SqlBoundKind, SqlBoundQuery};
use std::{cell::RefCell, rc::Rc};

impl Scope {
    pub(super) fn record_subquery(&self, bound: &SqlBoundQuery, query: &sqlparser::ast::Query) {
        use sqlparser::ast::Spanned;
        self.record_body(
            bound,
            || {
                let span = query.span();
                if span.start.line > 0 && span.start != span.end {
                    super::start(span)
                } else {
                    bound
                        .items
                        .first()
                        .map_or((1, 1), |item| (item.line, item.column))
                }
            },
            || "subquery".into(),
        );
    }

    pub(in crate::codebase::postgres) fn with_bodies(mut self, enabled: bool) -> Self {
        if enabled {
            self.enable_bodies();
        }
        self
    }

    pub(in crate::codebase::postgres) fn enable_bodies(&mut self) {
        self.bodies = Some(Rc::new(RefCell::new(Vec::new())));
    }

    pub(super) fn record_body(
        &self,
        query: &SqlBoundQuery,
        at: impl FnOnce() -> (usize, usize),
        name: impl FnOnce() -> String,
    ) {
        if let Some(bodies) = &self.bodies {
            let at = at();
            bodies.borrow_mut().push(SqlBoundFact {
                kind: SqlBoundKind::Select,
                body: Some(name()),
                line: at.0,
                column: at.1,
                statement_start: None,
                query: query.clone(),
                target: None,
            });
        }
    }

    pub(super) fn finish_bodies(&self, out: &mut Vec<SqlBoundFact>) {
        if let Some(bodies) = &self.bodies {
            let mut bodies = bodies.borrow_mut();
            bodies.sort_by_key(|body| (body.line, body.column));
            bodies.dedup_by_key(|body| (body.line, body.column));
            out.append(&mut bodies);
        }
    }
}

impl Scope {
    pub(super) fn expression_bodies(
        &self,
        select: &sqlparser::ast::Select,
        positions: super::super::value::PlaceholderPositions<'_>,
    ) {
        self.expressions_at(select, || select.select_token.0.span.start, positions);
    }

    pub(super) fn expressions_at<T: sqlparser::ast::Visit>(
        &self,
        expressions: &T,
        at: impl FnOnce() -> sqlparser::tokenizer::Location,
        positions: super::super::value::PlaceholderPositions<'_>,
    ) {
        if self.bodies.is_none() {
            return;
        }
        let at = at();
        let mut scope = self.clone();
        scope.table_tokens = self
            .table_tokens
            .as_ref()
            .map(|tokens| Rc::new(RefCell::new(tokens.borrow().for_body(at))));
        let mut visitor = Expressions {
            scope: &scope,
            positions,
            depth: 0,
        };
        let _ = expressions.visit(&mut visitor);
    }
}

struct Expressions<'a> {
    scope: &'a Scope,
    positions: super::super::value::PlaceholderPositions<'a>,
    depth: usize,
}

impl sqlparser::ast::Visitor for Expressions<'_> {
    type Break = ();

    fn pre_visit_query(&mut self, query: &sqlparser::ast::Query) -> std::ops::ControlFlow<()> {
        if self.depth == 0 {
            let bound = super::query::bound_query(query, self.scope, self.positions);
            self.scope.record_subquery(&bound, query);
        }
        self.depth += 1;
        std::ops::ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _: &sqlparser::ast::Query) -> std::ops::ControlFlow<()> {
        self.depth -= 1;
        std::ops::ControlFlow::Continue(())
    }
}
