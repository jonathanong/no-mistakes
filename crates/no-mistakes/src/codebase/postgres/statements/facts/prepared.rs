//! Prepared statements project facts at declaration and create destinations at execution.
use super::super::{bounds, SqlBoundFact};
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::Statement;
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct PreparedStatements<'a> {
    statements: BTreeMap<String, &'a Statement>,
}

impl<'a> PreparedStatements<'a> {
    pub(super) fn apply(
        &self,
        source: &Statement,
        executed: &Statement,
        temporary: &mut bounds::TemporaryRelations,
        facts: &mut [SqlBoundFact],
        scope: &bounds::Scope,
        positions: super::super::value::PlaceholderPositions<'_>,
    ) {
        if matches!(source, Statement::Prepare { .. }) {
            // PREPARE analyzes the query, but its SELECT INTO runs only at EXECUTE.
            temporary.clone().apply(executed, facts, scope, positions);
        } else {
            temporary.apply(executed, facts, scope, positions);
        }
        if let Statement::Execute {
            name: Some(name),
            immediate: false,
            ..
        } = executed
        {
            if let [part] = name.0.as_slice() {
                if let Some(statement) = part
                    .as_ident()
                    .and_then(|ident| self.statements.get(&ident_key(ident)))
                {
                    // Replay only the destination transition, without a second fact pass.
                    temporary.apply(statement, &mut [], scope, positions);
                }
            }
        }
    }

    pub(super) fn record(&mut self, source: &'a Statement) {
        match source {
            Statement::Prepare {
                name, statement, ..
            } => {
                self.statements.insert(ident_key(name), statement.as_ref());
            }
            Statement::Deallocate { name, .. }
                if name.quote_style.is_none() && name.value.eq_ignore_ascii_case("all") =>
            {
                self.statements.clear();
            }
            Statement::Deallocate { name, .. } => {
                self.statements.remove(&ident_key(name));
            }
            _ => {}
        }
    }
}
