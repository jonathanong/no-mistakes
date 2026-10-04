use super::super::{
    bounds::TableTokenCursor, SqlRelationPredicateFact, SqlSelectFact, SqlStarProjectionFact,
};
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{CopySource, Query, SetExpr, Spanned, Statement, Visit, Visitor};
use std::ops::ControlFlow;

/// Project TABLE query arms without changing their AST, which the bound and source
/// projections use to retain their established TABLE-specific behavior.
pub(in crate::codebase::postgres::statements) fn collect(
    statement: &Statement,
    cursor: Option<&mut TableTokenCursor>,
    out: &mut Vec<SqlSelectFact>,
) {
    let Some(cursor) = cursor else { return };
    match statement {
        Statement::Query(query) => query_arms(query, &[], false, false, cursor, out),
        Statement::Insert(insert) => {
            if let Some(query) = insert.source.as_deref() {
                query_arms(query, &[], true, false, cursor, out);
            }
        }
        Statement::Update(_) | Statement::Delete(_) => nested(statement, &[], false, cursor, out),
        Statement::CreateView(view) => query_arms(&view.query, &[], false, false, cursor, out),
        Statement::CreateTable(table) => {
            if let Some(query) = table.query.as_deref() {
                query_arms(query, &[], false, false, cursor, out);
            }
        }
        Statement::Copy {
            source: CopySource::Query(query),
            ..
        } => query_arms(query, &[], false, false, cursor, out),
        _ => {}
    }
}

fn query_arms(
    query: &Query,
    outer_ctes: &[String],
    in_insert_select: bool,
    in_exists: bool,
    cursor: &mut TableTokenCursor,
    out: &mut Vec<SqlSelectFact>,
) {
    let mut ctes = outer_ctes.to_vec();
    if let Some(with) = &query.with {
        if with.recursive {
            ctes.extend(with.cte_tables.iter().map(|cte| ident_key(&cte.alias.name)));
        }
        for cte in &with.cte_tables {
            let name = ident_key(&cte.alias.name);
            query_arms(&cte.query, &ctes, in_insert_select, false, cursor, out);
            if !with.recursive {
                ctes.push(name);
            }
        }
    }
    set_arms(&query.body, &ctes, in_insert_select, in_exists, cursor, out);
    nested(&query.order_by, &ctes, in_insert_select, cursor, out);
    nested(&query.limit_clause, &ctes, in_insert_select, cursor, out);
    nested(&query.fetch, &ctes, in_insert_select, cursor, out);
}

fn set_arms(
    expr: &SetExpr,
    ctes: &[String],
    in_insert_select: bool,
    in_exists: bool,
    cursor: &mut TableTokenCursor,
    out: &mut Vec<SqlSelectFact>,
) {
    match expr {
        SetExpr::Table(table) => {
            let Some(source) = cursor.take(table) else {
                return;
            };
            if table.schema_name.is_none() && ctes.contains(&source.key) {
                return;
            }
            let line = source.at.0.max(1);
            let relation = source.name;
            out.push(SqlSelectFact {
                line,
                tables: vec![relation.clone()],
                predicate_sql: String::new(),
                exists_set_operations: Vec::new(),
                relations: vec![SqlRelationPredicateFact {
                    table: relation.clone(),
                    alias: None,
                    constrained_columns: Vec::new(),
                    unqualified_columns: Vec::new(),
                    line,
                }],
                in_insert_select,
                not_in_subqueries: Vec::new(),
                not_in_columns: Vec::new(),
                count_existence_checks: Vec::new(),
                star_projections: (!in_exists)
                    .then_some(SqlStarProjectionFact {
                        relation,
                        qualified: false,
                        within_function: None,
                        line,
                    })
                    .into_iter()
                    .collect(),
                column_uses: Vec::new(),
            });
        }
        SetExpr::SetOperation { left, right, .. } => {
            set_arms(left, ctes, in_insert_select, in_exists, cursor, out);
            cursor.advance_to_right_arm(left.span().start);
            set_arms(right, ctes, in_insert_select, in_exists, cursor, out);
        }
        SetExpr::Query(query) => query_arms(query, ctes, in_insert_select, in_exists, cursor, out),
        SetExpr::Select(select) => nested(select.as_ref(), ctes, in_insert_select, cursor, out),
        SetExpr::Values(values) => nested(values, ctes, in_insert_select, cursor, out),
        SetExpr::Update(statement) | SetExpr::Delete(statement) => {
            nested(statement, ctes, in_insert_select, cursor, out);
        }
        SetExpr::Insert(Statement::Insert(insert)) => {
            if let Some(query) = insert.source.as_deref() {
                query_arms(query, ctes, true, false, cursor, out);
            }
        }
        _ => {}
    }
}

fn nested<T: Visit>(
    node: &T,
    ctes: &[String],
    in_insert_select: bool,
    cursor: &mut TableTokenCursor,
    out: &mut Vec<SqlSelectFact>,
) {
    let _ = node.visit(&mut Nested {
        ctes,
        in_insert_select,
        cursor,
        out,
        depth: 0,
        in_exists: false,
    });
}

struct Nested<'a> {
    ctes: &'a [String],
    in_insert_select: bool,
    cursor: &'a mut TableTokenCursor,
    out: &'a mut Vec<SqlSelectFact>,
    depth: usize,
    in_exists: bool,
}

impl Visitor for Nested<'_> {
    type Break = ();

    fn pre_visit_expr(&mut self, expr: &sqlparser::ast::Expr) -> ControlFlow<()> {
        if self.depth == 0 {
            self.in_exists = matches!(expr, sqlparser::ast::Expr::Exists { .. });
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        if self.depth == 0 {
            query_arms(
                query,
                self.ctes,
                self.in_insert_select,
                self.in_exists,
                self.cursor,
                self.out,
            );
        }
        self.depth += 1;
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.depth -= 1;
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mutation_tests;
