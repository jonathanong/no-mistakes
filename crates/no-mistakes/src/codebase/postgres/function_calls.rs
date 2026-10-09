//! Syntactic calls shared by SQL policies; names and locations retain SQL identity.
use super::idents::ident_key;
use sqlparser::ast::{
    Expr, ObjectName, OrderBy, Query, Select, Spanned, Statement, TableFactor, Visit, Visitor,
};
use std::ops::ControlFlow;

mod clause;
mod clauses;
pub use clause::SqlFunctionClause;

/// One function expression, excluding column and alias identifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlFunctionCallFact {
    pub name_parts: Vec<String>,
    pub line: usize,
    pub column: usize,
    /// None for calls outside the supported clauses, including FROM and GROUP BY.
    pub clause: Option<SqlFunctionClause>,
}

pub(super) fn collect(statement: &Statement, out: &mut Vec<SqlFunctionCallFact>) {
    let _ = statement.visit(&mut Calls {
        out,
        roots: clauses::Roots::default(),
        current: None,
        contexts: Vec::new(),
    });
}

struct Calls<'a> {
    out: &'a mut Vec<SqlFunctionCallFact>,
    roots: clauses::Roots,
    current: Option<SqlFunctionClause>,
    contexts: Vec<Option<SqlFunctionClause>>,
}

impl Calls<'_> {
    fn enter(&mut self, clause: Option<SqlFunctionClause>) {
        self.contexts.push(self.current);
        self.current = clause;
    }

    fn leave(&mut self) {
        self.current = self.contexts.pop().expect("balanced SQL visitor context");
    }
}

impl Visitor for Calls<'_> {
    type Break = ();

    fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<()> {
        self.enter(None);
        self.roots.statement(statement);
        ControlFlow::Continue(())
    }

    fn post_visit_statement(&mut self, _statement: &Statement) -> ControlFlow<()> {
        self.leave();
        ControlFlow::Continue(())
    }

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        // A subquery owns its clauses even when it occurs inside an outer expression.
        self.enter(None);
        self.roots.query(query);
        ControlFlow::Continue(())
    }

    fn post_visit_query(&mut self, _query: &Query) -> ControlFlow<()> {
        self.leave();
        ControlFlow::Continue(())
    }

    fn pre_visit_select(&mut self, select: &Select) -> ControlFlow<()> {
        self.roots.select(select);
        ControlFlow::Continue(())
    }

    fn pre_visit_order_by(&mut self, ordering: &OrderBy) -> ControlFlow<()> {
        self.roots.order_by(ordering);
        ControlFlow::Continue(())
    }

    fn pre_visit_expr(&mut self, expression: &Expr) -> ControlFlow<()> {
        self.enter(self.roots.clause(expression).or(self.current));
        if let Expr::Function(function) = expression {
            self.roots.function(function);
            record(&function.name, self.current, self.out);
        }
        ControlFlow::Continue(())
    }

    fn post_visit_expr(&mut self, _expression: &Expr) -> ControlFlow<()> {
        self.leave();
        ControlFlow::Continue(())
    }

    fn pre_visit_table_factor(&mut self, table: &TableFactor) -> ControlFlow<()> {
        match table {
            TableFactor::Table {
                name,
                args: Some(_),
                ..
            }
            | TableFactor::Function { name, .. } => record(name, self.current, self.out),
            _ => {}
        }
        ControlFlow::Continue(())
    }
}

fn record(
    name: &ObjectName,
    clause: Option<SqlFunctionClause>,
    out: &mut Vec<SqlFunctionCallFact>,
) {
    let parts: Option<Vec<_>> = name
        .0
        .iter()
        .map(|part| part.as_ident().map(ident_key))
        .collect();
    if let Some(name_parts) = parts {
        let start = name.span().start;
        out.push(SqlFunctionCallFact {
            name_parts,
            line: start.line.max(1) as usize,
            column: start.column.max(1) as usize,
            clause,
        });
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod clause_tests;

#[cfg(test)]
mod join_tests;

#[cfg(test)]
mod variant_tests;

#[cfg(test)]
mod mutation_tests;
