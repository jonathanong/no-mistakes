use super::{
    expressions::{identifier, name},
    locations::Locations,
    types::*,
};
use sqlparser::ast::{Query, Spanned};
use std::collections::{BTreeMap, BTreeSet};
mod bodies;
mod clauses;
mod ctes;
mod predicates;
mod relations;
mod resolution;
mod statements;

type CteEnvironment = BTreeMap<String, usize>;
struct ScopeState {
    visible_parent: Option<usize>,
    names: BTreeMap<Vec<String>, Vec<usize>>,
}
struct Collector<'a, 's> {
    locations: &'a Locations<'s>,
    facts: PostgresSqlQuery,
    states: Vec<ScopeState>,
    depth: usize,
}

pub(super) fn project(query: &Query, locations: &Locations<'_>) -> PostgresSqlQuery {
    let mut collector = Collector {
        locations,
        facts: PostgresSqlQuery {
            complete: true,
            ..Default::default()
        },
        states: Vec::new(),
        depth: 0,
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
        let scope = self.scope(
            parent,
            visible_parent,
            clause,
            definition,
            self.locations.span(query.span()),
        );
        // Bound projection independently of parser configuration for library AST callers.
        if self.depth == 64 {
            self.unsupported(scope, clause, "query nesting limit", query.span());
            return scope;
        }
        self.depth += 1;
        let env = self.ctes(query, scope, outer);
        self.body(&query.body, scope, &env);
        self.query_clauses(query, scope, &env);
        self.depth -= 1;
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
            ..Default::default()
        }
    }
}
