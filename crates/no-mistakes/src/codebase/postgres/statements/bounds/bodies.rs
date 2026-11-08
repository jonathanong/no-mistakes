use super::{Scope, SqlBoundFact, SqlBoundKind, SqlBoundQuery};
use crate::codebase::postgres::statements::SqlBoundItem;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};
mod expressions;
mod pools;

#[derive(Default)]
pub(super) struct Collected {
    facts: Vec<SqlBoundFact>,
    references: BTreeSet<(usize, usize)>,
    // Retained canonical CTEs prevent address reuse in request-local identity keys.
    pools: BTreeMap<usize, Rc<SqlBoundQuery>>,
}

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
        self.bodies = Some(Rc::default());
    }

    pub(super) fn record_body(
        &self,
        query: &SqlBoundQuery,
        at: impl FnOnce() -> (usize, usize),
        name: impl FnOnce() -> String,
    ) {
        if let Some(bodies) = &self.bodies {
            let at = at();
            let mut bodies = bodies.borrow_mut();
            let query = pools::project(query, &bodies.references);
            bodies.facts.push(SqlBoundFact {
                kind: self.body_owner.unwrap_or(SqlBoundKind::Select),
                body: Some(name()),
                line: at.0,
                column: at.1,
                statement_start: None,
                query,
                target: None,
            });
        }
    }

    pub(super) fn finish_bodies(&self, out: &mut Vec<SqlBoundFact>) {
        if let Some(bodies) = &self.bodies {
            let mut collected = bodies.borrow_mut();
            let bodies = &mut collected.facts;
            bodies.sort_by_key(|body| (body.line, body.column));
            bodies.dedup_by_key(|body| (body.line, body.column));
            out.append(bodies);
        }
    }
}

impl Scope {
    pub(super) fn expression_bodies(
        &self,
        select: &sqlparser::ast::Select,
        items: &[SqlBoundItem],
        positions: super::super::value::PlaceholderPositions<'_>,
    ) {
        self.expressions_at(
            select,
            items,
            || select.select_token.0.span.start,
            positions,
        );
    }

    pub(super) fn expressions_at<T: sqlparser::ast::Visit>(
        &self,
        expressions: &T,
        items: &[SqlBoundItem],
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
        expressions::collect(expressions, &scope, items, positions);
    }
}
