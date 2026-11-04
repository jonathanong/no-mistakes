use super::{functions, Scope, TableTokenCursor};
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{visit_expressions, Expr, ObjectName, Statement};
use std::collections::BTreeSet;
use std::ops::ControlFlow;
use std::sync::Arc;

impl Scope {
    pub(in super::super) fn table_cursor(&self) -> Option<TableTokenCursor> {
        self.table_tokens
            .as_ref()
            .map(|cursor| cursor.borrow().clone())
    }

    pub(in super::super) fn with_scalar_functions(
        cursor: Option<TableTokenCursor>,
        functions: Arc<BTreeSet<Vec<String>>>,
    ) -> Self {
        let mut scope = cursor.map(Self::with_table_tokens).unwrap_or_default();
        scope.scalar_functions = functions;
        scope
    }

    pub(super) fn scalar_function(&self, name: &ObjectName) -> bool {
        let names = name
            .0
            .iter()
            .map(|part| part.as_ident().map(ident_key))
            .collect::<Option<Vec<_>>>();
        names.is_some_and(|names| self.scalar_functions.contains(&names))
    }
}

pub(in super::super) fn needs_catalog(statement: &Statement) -> bool {
    visit_expressions(statement, |expression| {
        if matches!(expression, Expr::Function(function) if functions::projection_can_expand(function) && !functions::is_set_returning(&function.name)) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }).is_break()
}
