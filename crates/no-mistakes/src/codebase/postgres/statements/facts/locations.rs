//! Token provenance projected from the AST already owned by the statement pass.
mod columns;
mod nodes;
mod project;
mod stars;
mod writes;
use super::super::{SqlColumnClause, SqlStatementFileFacts, SqlVariantLocations};
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
    pub(super) fn new(binds: &[(u32, u32)]) -> Self {
        Self {
            binds: binds.to_vec(),
            ..Default::default()
        }
    }
    pub(super) fn collect(&mut self, statement: &Statement) {
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
        project::collect(self, facts, tokens)
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
) -> SqlVariantLocations {
    let mut projected = locations.project(facts, tokens);
    match crate::codebase::postgres::conflict::analyze_parsed(sql, statements, binds) {
        Ok(conflicts) => projected.conflicts = conflicts,
        Err(error) => projected.conflict_error = Some(error.to_string()),
    }
    projected
}
