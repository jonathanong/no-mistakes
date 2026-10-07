//! One completeness definition for nested statement source occurrences.
use super::types::*;

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
        PostgresSqlStatementKind::DoBlock { block } => block.complete,
        PostgresSqlStatementKind::Conditional { branches } => branches.iter().all(|branch| {
            branch
                .statements
                .iter()
                .all(|value| statement(&value.facts))
        }),
        PostgresSqlStatementKind::Other => false,
        _ => true,
    }
}
