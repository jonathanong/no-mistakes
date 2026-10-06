//! Syntactic calls shared by SQL policies; names and locations retain SQL identity.
use super::idents::ident_key;
use sqlparser::ast::{Expr, ObjectName, Spanned, Statement, TableFactor, Visit, Visitor};
use std::ops::ControlFlow;

/// One function expression, excluding column and alias identifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlFunctionCallFact {
    pub name_parts: Vec<String>,
    pub line: usize,
    pub column: usize,
}

pub(super) fn collect(statement: &Statement, out: &mut Vec<SqlFunctionCallFact>) {
    let _ = statement.visit(&mut Calls(out));
}

struct Calls<'a>(&'a mut Vec<SqlFunctionCallFact>);

impl Visitor for Calls<'_> {
    type Break = ();

    fn pre_visit_expr(&mut self, expression: &Expr) -> ControlFlow<()> {
        if let Expr::Function(function) = expression {
            record(&function.name, self.0);
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_table_factor(&mut self, table: &TableFactor) -> ControlFlow<()> {
        match table {
            TableFactor::Table {
                name,
                args: Some(_),
                ..
            }
            | TableFactor::Function { name, .. } => record(name, self.0),
            _ => {}
        }
        ControlFlow::Continue(())
    }
}

fn record(name: &ObjectName, out: &mut Vec<SqlFunctionCallFact>) {
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
        });
    }
}

#[cfg(test)]
mod tests;
