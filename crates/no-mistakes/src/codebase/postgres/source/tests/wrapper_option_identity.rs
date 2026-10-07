use super::facts;
use crate::codebase::postgres::source::*;

#[test]
fn explain_option_aliases_and_quoted_values_match_postgresql_identity() {
    let result = facts("wrapper-option-identity.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 18, "{:?}", result.statements);
    use PostgresSqlExecution::{ExecutesForAnalysis as Runs, NonExecuting as Plan, Unknown};
    for (statement, expected) in result.statements.iter().zip([
        Runs, Plan, Runs, Plan, Plan, Plan, Plan, Unknown, Runs, Runs, Unknown, Plan, Unknown,
        Unknown, Unknown, Unknown, Unknown,
    ]) {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &statement.facts else {
            panic!("EXPLAIN expected")
        };
        assert_eq!(wrapper.execution, expected, "{}", statement.sql);
        assert_eq!(wrapper.complete, expected != Unknown, "{}", statement.sql);
    }
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[17].facts else {
        panic!("DO expected")
    };
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!("IF expected")
    };
    for statement in &branches[0].statements {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &statement.facts else {
            panic!("EXPLAIN expected")
        };
        assert!(!wrapper.complete);
        let columns = match &wrapper.statements[0].facts {
            PostgresSqlStatementKind::CreateTable { columns, .. } => columns.as_slice(),
            PostgresSqlStatementKind::AlterTable { operations, .. } => {
                let PostgresSqlAlterOperation::AddColumn { column, .. } = &operations[0] else {
                    panic!("ADD COLUMN expected")
                };
                std::slice::from_ref(column.as_ref())
            }
            _ => panic!("generated-column child expected"),
        };
        assert_eq!(
            columns
                .last()
                .unwrap()
                .generated
                .as_ref()
                .unwrap()
                .storage
                .as_deref(),
            Some("VIRTUAL")
        );
    }
}
