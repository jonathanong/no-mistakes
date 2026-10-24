//! Token provenance projected from the AST already owned by the statement pass.
mod columns;
mod nodes;
mod project;
mod stars;
mod writes;
use super::super::{SqlColumnClause, SqlStatementFileFacts, SqlVariantLocations};
use crate::codebase::postgres::statement_facts::SqlLocationDemand;
use sqlparser::ast::{Expr, Query, Select, SetExpr, Statement, TableFactor, Visit, Visitor};
use sqlparser::tokenizer::{Span, TokenWithSpan};
use std::ops::ControlFlow;

#[derive(Clone)]
struct Relation {
    table: String,
    alias: Option<String>,
    at: (usize, usize),
}
#[derive(Clone)]
struct Star {
    qualifier: Option<String>,
    function: Option<String>,
    at: (usize, usize),
}
#[derive(Clone)]
struct Column {
    name: String,
    qualifier: Option<String>,
    clause: SqlColumnClause,
    at: (usize, usize),
}
struct Selection {
    at: (usize, usize),
    predicate: String,
    relations: Vec<Relation>,
    stars: Vec<Star>,
    columns: Vec<Column>,
}
#[derive(Default)]
pub(super) struct Locations {
    demand: SqlLocationDemand,
    selections: Vec<Selection>,
    relations: Vec<Relation>,
    returning: Vec<(Star, Vec<Relation>)>,
    mutation_columns: Vec<Column>,
    inserts: Vec<(String, (usize, usize))>,
    writes: Vec<writes::Write>,
    pub(super) locking: Vec<(
        crate::codebase::postgres::LockingSelectMetadata,
        sqlparser::tokenizer::Span,
    )>,
    pub(super) binds: Vec<(u32, u32)>,
}
impl Locations {
    pub(super) fn new(binds: &[(u32, u32)], demand: SqlLocationDemand) -> Self {
        Self {
            demand,
            binds: binds.to_vec(),
            ..Default::default()
        }
    }
    pub(super) fn collect(&mut self, statement: &Statement) {
        if self.demand != SqlLocationDemand::Full {
            return;
        }
        work::full_walk();
        self.locking
            .extend(crate::codebase::postgres::locking::collect_located(
                statement,
                &self.binds,
            ));
        let _: ControlFlow<()> = statement.visit(self);
    }
    pub(super) fn project(
        &self,
        facts: &SqlStatementFileFacts,
        tokens: &[TokenWithSpan],
    ) -> SqlVariantLocations {
        if self.demand == SqlLocationDemand::Full {
            project::collect(self, facts, tokens)
        } else {
            project::conflicts(tokens)
        }
    }
    fn selections(&mut self, body: &SetExpr, order: &[sqlparser::ast::OrderByExpr]) {
        match body {
            SetExpr::Select(select) => self.selection(select, order),
            SetExpr::SetOperation { left, right, .. } => {
                self.selections(left, &[]);
                self.selections(right, &[]);
            }
            _ => {}
        }
    }
    fn selection(&mut self, select: &Select, order: &[sqlparser::ast::OrderByExpr]) {
        let relations = nodes::relations(&select.from);
        self.selections.push(Selection {
            at: at(select.select_token.0.span),
            predicate: super::super::select::predicate_text(select),
            stars: stars::items(&select.projection),
            columns: columns::selection(select, order),
            relations,
        });
    }
}
impl Visitor for Locations {
    type Break = ();
    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        let order = match query.order_by.as_ref().map(|order| &order.kind) {
            Some(sqlparser::ast::OrderByKind::Expressions(exprs)) => exprs.as_slice(),
            _ => &[],
        };
        self.selections(&query.body, order);
        ControlFlow::Continue(())
    }
    fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
        if let Some(relation) = nodes::relation(factor) {
            self.relations.push(relation);
        }
        ControlFlow::Continue(())
    }
    fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<()> {
        nodes::statement(self, statement);
        ControlFlow::Continue(())
    }
}
fn at(span: Span) -> (usize, usize) {
    (
        span.start.line.max(1) as usize,
        span.start.column.max(1) as usize,
    )
}
fn bare(expr: &Expr) -> Option<(Option<String>, String)> {
    let mut expr = expr;
    while let Expr::Nested(inner) = expr {
        expr = inner;
    }
    match expr {
        Expr::Identifier(name) => Some((None, crate::codebase::postgres::idents::ident_key(name))),
        Expr::CompoundIdentifier(names) => Some((
            Some(
                names[..names.len() - 1]
                    .iter()
                    .map(crate::codebase::postgres::idents::ident_key)
                    .collect::<Vec<_>>()
                    .join("."),
            ),
            crate::codebase::postgres::idents::ident_key(names.last()?),
        )),
        _ => None,
    }
}

pub(super) fn finish(
    locations: Locations,
    facts: &SqlStatementFileFacts,
    tokens: &[TokenWithSpan],
    sql: &str,
    statements: &[Statement],
    binds: &[(u32, u32)],
    recovery: Option<&crate::codebase::postgres::conflict::ConflictChunks<'_>>,
) -> SqlVariantLocations {
    let mut projected = locations.project(facts, tokens);
    for (index, failure) in facts.parse_failures.iter().enumerate() {
        projected.insert(
            super::SqlFactSite::ParseFailure(index),
            failure.line,
            failure.column,
        );
    }
    match crate::codebase::postgres::conflict::analyze_parsed(sql, statements, binds) {
        Ok(conflicts) => projected.conflicts = conflicts,
        Err(error) => projected.conflict_error = Some(error.to_string()),
    }
    if facts.parse_failed {
        // A sibling chunk can fail while this conflict statement parsed completely.
        // Only a failure inside the conflict statement, or the CTE that encloses it, abstains.
        let recovery = crate::codebase::postgres::conflict::reliable_chunk_tokens(
            statements,
            binds,
            recovery.unwrap_or(&crate::codebase::postgres::conflict::ConflictChunks::EMPTY),
        );
        projected.reliable_conflicts = recovery.reliable;
        projected.conflict_failure_overlaps = recovery.failure_overlaps_conflict;
    }
    projected
}

#[cfg(test)]
pub(crate) mod work;

#[cfg(not(test))]
#[path = "locations/no_work.rs"]
pub(crate) mod work;
