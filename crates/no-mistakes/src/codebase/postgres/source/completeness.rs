//! One completeness definition for nested statement source occurrences.
use super::types::*;
use std::collections::BTreeSet;

pub(super) type UtilitySpans = BTreeSet<(usize, usize)>;

pub(super) fn statement(facts: &PostgresSqlStatementKind) -> bool {
    match facts {
        PostgresSqlStatementKind::Insert { insert } => insert.complete,
        PostgresSqlStatementKind::Select { query } => query.complete,
        PostgresSqlStatementKind::AlterTable { operations, .. } => {
            operations.iter().all(|operation| match operation {
                PostgresSqlAlterOperation::Other { .. } => false,
                PostgresSqlAlterOperation::AddConstraint { constraint, .. } => {
                    constraint.kind != PostgresSqlConstraintKind::Other
                }
                _ => true,
            })
        }
        PostgresSqlStatementKind::LiteralExecute { execute } => execute.complete,
        PostgresSqlStatementKind::DoBlock { block } => block.complete,
        PostgresSqlStatementKind::Conditional { branches } => branches.iter().all(|branch| {
            branch
                .statements
                .iter()
                .all(|value| statement(&value.facts))
        }),
        PostgresSqlStatementKind::Wrapper { wrapper } => wrapper.complete,
        PostgresSqlStatementKind::CreateFunction { function } => function.wrapper.complete,
        PostgresSqlStatementKind::CreateView { view } => view.dependencies_complete,
        PostgresSqlStatementKind::Other => false,
        _ => true,
    }
}

pub(super) fn procedural_statements(
    statements: &[PostgresSqlStatement],
    utilities: &UtilitySpans,
) -> bool {
    statements
        .iter()
        .all(|statement| procedural_statement(statement, utilities))
}

fn procedural_statement(statement: &PostgresSqlStatement, utilities: &UtilitySpans) -> bool {
    match &statement.facts {
        PostgresSqlStatementKind::Other => {
            utilities.contains(&(statement.span.start.offset, statement.span.end.offset))
        }
        PostgresSqlStatementKind::Conditional { branches } => branches
            .iter()
            .all(|branch| procedural_statements(&branch.statements, utilities)),
        facts => self::statement(facts),
    }
}
