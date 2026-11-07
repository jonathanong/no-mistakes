//! Project predicate transformations from the request's already parsed SQL AST.
mod boolean;
mod cast_sites;
mod cte_scope;
mod expressions;
mod scope;
#[cfg(test)]
mod tests;
use super::value::PlaceholderPositions;
use super::{SqlTransformedColumnClause as Clause, SqlTransformedColumnUseFact};
use cte_scope::QueryScopes;
use scope::base_relations;
use sqlparser::ast::{
    Expr, JoinOperator, Query, Select, SetExpr, Statement, TableFactor, TableWithJoins, Visit,
    Visitor,
};
use std::ops::ControlFlow;

pub(super) fn collect(
    statement: &Statement,
    positions: PlaceholderPositions<'_>,
    tokens: &[sqlparser::tokenizer::TokenWithSpan],
    out: &mut Vec<SqlTransformedColumnUseFact>,
) {
    let _: ControlFlow<()> = statement.visit(&mut Collector {
        ctes: QueryScopes::default(),
        positions,
        tokens,
        out,
    });
}

struct Collector<'a, 'b> {
    ctes: QueryScopes,
    positions: PlaceholderPositions<'a>,
    tokens: &'a [sqlparser::tokenizer::TokenWithSpan],
    out: &'b mut Vec<SqlTransformedColumnUseFact>,
}
impl Collector<'_, '_> {
    fn body(&mut self, body: &SetExpr, ctes: &[String]) {
        match body {
            SetExpr::Select(select) => self.select(select, ctes),
            SetExpr::SetOperation { left, right, .. } => {
                self.body(left, ctes);
                self.body(right, ctes);
            }
            _ => {}
        }
    }
    fn select(&mut self, select: &Select, ctes: &[String]) {
        self.predicates(&select.from, select.selection.as_ref(), ctes);
        if let Some(having) = &select.having {
            expressions::collect(
                having,
                Clause::Having,
                &base_relations(&select.from, ctes),
                None,
                self.positions,
                self.tokens,
                self.out,
            );
        }
    }
    fn predicates(&mut self, tables: &[TableWithJoins], selection: Option<&Expr>, ctes: &[String]) {
        if let Some(selection) = selection {
            expressions::collect(
                selection,
                Clause::Where,
                &base_relations(tables, ctes),
                None,
                self.positions,
                self.tokens,
                self.out,
            );
        }
        // ON sees preceding factors in its own join chain, never later joins.
        for table in tables {
            self.nested_joins(&table.relation, ctes);
            let mut visible = Vec::new();
            scope::push_factor(&table.relation, ctes, &mut visible);
            for join in &table.joins {
                self.nested_joins(&join.relation, ctes);
                let mut probed = Vec::new();
                scope::push_factor(&join.relation, ctes, &mut probed);
                scope::push_factor(&join.relation, ctes, &mut visible);
                let probe_scope = matches!(
                    &join.join_operator,
                    JoinOperator::Join(_)
                        | JoinOperator::Inner(_)
                        | JoinOperator::Left(_)
                        | JoinOperator::LeftOuter(_)
                )
                .then_some(probed.as_slice());
                if let Some(expr) = super::select::join_expr(&join.join_operator) {
                    expressions::collect(
                        expr,
                        Clause::Join,
                        &visible,
                        probe_scope,
                        self.positions,
                        self.tokens,
                        self.out,
                    );
                }
            }
        }
    }
    fn nested_joins(&mut self, factor: &TableFactor, ctes: &[String]) {
        if let TableFactor::NestedJoin {
            table_with_joins, ..
        } = factor
        {
            self.predicates(std::slice::from_ref(table_with_joins), None, ctes);
        }
    }
}
impl Visitor for Collector<'_, '_> {
    type Break = ();
    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        self.ctes.enter(query);
        let ctes = self.ctes.visible().to_vec();
        self.body(&query.body, &ctes);
        ControlFlow::Continue(())
    }
    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.ctes.exit();
        ControlFlow::Continue(())
    }
    fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<()> {
        let ctes = self.ctes.visible().to_vec();
        match statement {
            Statement::Update(update) => {
                let mut tables = vec![update.table.clone()];
                if let Some(from) = &update.from {
                    match from {
                        sqlparser::ast::UpdateTableFromKind::BeforeSet(from)
                        | sqlparser::ast::UpdateTableFromKind::AfterSet(from) => {
                            tables.extend(from.iter().cloned())
                        }
                    }
                }
                self.predicates(&tables, update.selection.as_ref(), &ctes);
            }
            Statement::Delete(delete) => {
                let mut tables = match &delete.from {
                    sqlparser::ast::FromTable::WithFromKeyword(from)
                    | sqlparser::ast::FromTable::WithoutKeyword(from) => from.clone(),
                };
                if let Some(using) = &delete.using {
                    tables.extend(using.iter().cloned());
                }
                self.predicates(&tables, delete.selection.as_ref(), &ctes);
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}
