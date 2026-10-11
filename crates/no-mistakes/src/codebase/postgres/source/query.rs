use super::{
    expressions::{identifier, name},
    locations::Locations,
    types::*,
};
use sqlparser::ast::{Query, Spanned};
use sqlparser::tokenizer::TokenWithSpan;
use std::collections::{BTreeMap, BTreeSet};
mod bodies;
mod clauses;
mod ctes;
mod predicates;
mod relations;
mod resolution;
mod span_bounds;
mod statement_bounds;
mod statements;

use span_bounds::QuerySpanBounds;

type CteEnvironment = BTreeMap<String, usize>;
struct ScopeState {
    visible_parent: Option<usize>,
    names: BTreeMap<Vec<String>, Vec<usize>>,
}
struct Collector<'a, 's> {
    locations: &'a Locations<'s>,
    /// Prepared tokens for the statement that owns this query. Never reparsed.
    tokens: &'a [TokenWithSpan],
    facts: PostgresSqlQuery,
    states: Vec<ScopeState>,
    depth: usize,
    /// NOT operators wrapping the expression currently being walked.
    /// Reset at each query boundary; not part of the serialized facts.
    not_depth: u32,
    insert_source: bool,
    span_bounds: QuerySpanBounds,
}

pub(super) fn project(
    query: &Query,
    locations: &Locations<'_>,
    tokens: &[TokenWithSpan],
) -> PostgresSqlQuery {
    let mut collector = Collector {
        locations,
        tokens,
        facts: PostgresSqlQuery {
            complete: true,
            ..Default::default()
        },
        states: Vec::new(),
        depth: 0,
        not_depth: 0,
        insert_source: false,
        span_bounds: QuerySpanBounds::new(tokens, locations),
    };
    collector.query(
        query,
        None,
        None,
        PostgresSqlQueryClause::Root,
        &CteEnvironment::new(),
        None,
    );
    collector.finish_ctes();
    collector
        .facts
        .nested_statements
        .sort_by_key(|statement| statement.span.as_ref().map(|span| span.start.offset));
    for (ordinal, statement) in collector.facts.nested_statements.iter_mut().enumerate() {
        statement.ordinal = ordinal;
    }
    collector.facts
}

impl Collector<'_, '_> {
    /// Prepared tokens that overlap one nested statement, in parser order.
    fn statement_tokens(&self, span: sqlparser::tokenizer::Span) -> &[TokenWithSpan] {
        let start = self
            .tokens
            .partition_point(|token| token.span.end <= span.start);
        let end = start + self.tokens[start..].partition_point(|token| token.span.start < span.end);
        &self.tokens[start..end]
    }

    fn scope(
        &mut self,
        parent: Option<usize>,
        visible_parent: Option<usize>,
        clause: PostgresSqlQueryClause,
        definition: Option<usize>,
        span: Option<PostgresSqlSpan>,
    ) -> usize {
        let id = self.facts.scopes.len();
        self.facts.scopes.push(PostgresSqlQueryScope {
            id,
            parent_scope_id: parent,
            clause,
            cte_definition_id: definition,
            set_operation: None,
            set_quantifier: None,
            span,
        });
        self.states.push(ScopeState {
            visible_parent,
            names: BTreeMap::new(),
        });
        id
    }
    fn query(
        &mut self,
        query: &Query,
        parent: Option<usize>,
        visible_parent: Option<usize>,
        clause: PostgresSqlQueryClause,
        outer: &CteEnvironment,
        definition: Option<usize>,
    ) -> usize {
        self.query_with_span(
            query,
            (parent, visible_parent),
            clause,
            outer,
            definition,
            None,
        )
    }
    fn query_with_span(
        &mut self,
        query: &Query,
        (parent, visible_parent): (Option<usize>, Option<usize>),
        clause: PostgresSqlQueryClause,
        outer: &CteEnvironment,
        definition: Option<usize>,
        preferred_span: Option<PostgresSqlSpan>,
    ) -> usize {
        let span = preferred_span.or_else(|| {
            self.span_bounds
                .query(query, parent.is_none(), self.locations)
                .or_else(|| self.locations.span(query.span()))
        });
        let scope = self.scope(parent, visible_parent, clause, definition, span);
        // Outer NOT does not apply inside a nested query. Restore even when nesting stops.
        let outer_not_depth = std::mem::take(&mut self.not_depth);
        // Bound projection independently of parser configuration for library AST callers.
        if self.depth == 64 {
            self.not_depth = outer_not_depth;
            self.unsupported(scope, clause, "query nesting limit", query.span());
            return scope;
        }
        self.depth += 1;
        let env = self.ctes(query, scope, outer);
        let body_end = self.span_bounds.body_end(
            query,
            self.facts.scopes[scope].span.as_ref(),
            self.locations,
        );
        self.body(&query.body, scope, &env, body_end);
        self.query_clauses(query, scope, &env);
        self.depth -= 1;
        self.not_depth = outer_not_depth;
        scope
    }
    fn unsupported(
        &mut self,
        scope: usize,
        clause: PostgresSqlQueryClause,
        reason: &str,
        span: sqlparser::tokenizer::Span,
    ) {
        self.facts.complete = false;
        self.facts.unsupported.push(PostgresSqlQueryUnsupported {
            scope_id: scope,
            clause,
            reason: reason.into(),
            span: self.locations.span(span),
        });
    }
    fn predicate_context(mandatory: bool) -> PostgresSqlPredicateContext {
        PostgresSqlPredicateContext {
            mandatory,
            effective_mandatory: Some(mandatory),
            ..Default::default()
        }
    }
}
