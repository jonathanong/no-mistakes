//! Prepared statements project facts at declaration and create destinations at execution.
use super::super::{bounds, SqlBoundFact};
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{visit_expressions, DiscardObject, Expr, Statement, Value};
use std::collections::BTreeMap;
use std::ops::ControlFlow;

struct Prepared<'a> {
    statement: &'a Statement,
    parameter_count: Option<usize>,
}

#[derive(Default)]
pub(super) struct PreparedStatements<'a> {
    statements: BTreeMap<String, Prepared<'a>>,
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
            parameters,
            ..
        } = executed
        {
            if let [part] = name.0.as_slice() {
                if let Some(prepared) = part
                    .as_ident()
                    .and_then(|ident| self.statements.get(&ident_key(ident)))
                {
                    if prepared.parameter_count == Some(parameters.len()) {
                        // Replay only the destination transition, without a second fact pass.
                        temporary.apply(prepared.statement, &mut [], scope, positions);
                    }
                }
            }
        }
    }

    pub(super) fn record(&mut self, source: &'a Statement) {
        match source {
            Statement::Prepare {
                name,
                data_types,
                statement,
            } => {
                // PostgreSQL rejects a duplicate name and retains its original definition.
                self.statements
                    .entry(ident_key(name))
                    .or_insert_with(|| Prepared {
                        statement,
                        parameter_count: parameter_count(statement, data_types.len()),
                    });
            }
            Statement::Deallocate { name, .. }
                if name.quote_style.is_none() && name.value.eq_ignore_ascii_case("all") =>
            {
                self.statements.clear();
            }
            Statement::Deallocate { name, .. } => {
                self.statements.remove(&ident_key(name));
            }
            Statement::Discard {
                object_type: DiscardObject::ALL,
                ..
            } => {
                self.statements.clear();
            }
            _ => {}
        }
    }
}

fn parameter_count(statement: &Statement, declared: usize) -> Option<usize> {
    let mut inferred = 0;
    let mut valid = true;
    let _: ControlFlow<()> = visit_expressions(statement, |expression| {
        if let Expr::Value(value) = expression {
            if let Value::Placeholder(placeholder) = &value.value {
                if let Some(number) = placeholder
                    .strip_prefix('$')
                    .and_then(|number| number.parse::<usize>().ok())
                    .filter(|number| *number > 0)
                {
                    inferred = inferred.max(number);
                } else {
                    valid = false;
                }
            }
        }
        ControlFlow::Continue(())
    });
    (valid && (declared == 0 || inferred <= declared)).then_some(declared.max(inferred))
}
