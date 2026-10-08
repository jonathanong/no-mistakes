//! Preserve an outer WITH in the existing INSERT SELECT query facts.
use super::ConflictFacts;
use sqlparser::ast::{SetExpr, Spanned, Statement};

pub(super) fn normalize(statement: Statement) -> (Statement, Option<ConflictFacts>) {
    let mut outer = match statement {
        Statement::Explain {
            describe_alias,
            analyze,
            verbose,
            query_plan,
            estimate,
            statement,
            format,
            options,
        } => {
            let (statement, facts) = normalize(*statement);
            return (
                Statement::Explain {
                    describe_alias,
                    analyze,
                    verbose,
                    query_plan,
                    estimate,
                    statement: Box::new(statement),
                    format,
                    options,
                },
                facts,
            );
        }
        Statement::Prepare {
            name,
            data_types,
            statement,
        } => {
            let (statement, facts) = normalize(*statement);
            return (
                Statement::Prepare {
                    name,
                    data_types,
                    statement: Box::new(statement),
                },
                facts,
            );
        }
        Statement::Query(outer) => outer,
        _ => return (statement, None),
    };
    let SetExpr::Insert(Statement::Insert(mut insert)) = *outer.body else {
        return (Statement::Query(outer), None);
    };
    let mut facts = ConflictFacts {
        expressions: Vec::new(),
        assignments: Vec::new(),
        predicate: None,
        action_predicate: None,
        span: None,
        source_span: None,
        unsupported_with: true,
    };
    if let Some(source) = &mut insert.source {
        facts.source_span = Some(source.span());
        if source.with.is_none()
            && matches!(
                source.body.as_ref(),
                SetExpr::Select(_) | SetExpr::Query(_) | SetExpr::SetOperation { .. }
            )
        {
            source.with = outer.with.take();
            facts.unsupported_with = false;
        }
    }
    (Statement::Insert(insert), Some(facts))
}
