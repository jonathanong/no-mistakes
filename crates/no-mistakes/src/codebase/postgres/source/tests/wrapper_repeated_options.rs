use super::{facts, fixture};
use crate::codebase::postgres::source::*;

#[test]
fn repeated_options_validate_each_occurrence_and_use_final_values() {
    let result = facts("wrapper-repeated-options.sql");
    let sql = fixture("wrapper-repeated-options.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 28);
    use PostgresSqlExecution::{ExecutesForAnalysis as Runs, NonExecuting as Plan, Unknown};
    for (statement, expected) in result.statements.iter().zip([
        Plan, Runs, Plan, Runs, Runs, Unknown, Plan, Unknown, Plan, Unknown, Plan, Unknown, Plan,
        Runs, Plan, Plan, Unknown, Unknown, Unknown, Unknown, Unknown, Unknown, Unknown, Plan,
        Plan,
    ]) {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &statement.facts else {
            panic!("EXPLAIN expected")
        };
        assert_eq!(wrapper.execution, expected, "{}", statement.sql);
        assert_eq!(wrapper.complete, expected != Unknown, "{}", statement.sql);
        assert_eq!(wrapper.diagnostics.is_empty(), expected != Unknown);
        assert_eq!(wrapper.statements.len(), 1);
        let child = &wrapper.statements[0];
        assert_eq!(
            child.sql,
            sql[child.span.start.offset..child.span.end.offset]
        );
        assert!(matches!(
            child.facts,
            PostgresSqlStatementKind::Select { .. }
        ));
    }
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[25].facts else {
        panic!("DO expected")
    };
    assert!(block.complete);
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!("IF expected")
    };
    let PostgresSqlStatementKind::Wrapper { wrapper } = &branches[0].statements[0].facts else {
        panic!("conditional EXPLAIN expected")
    };
    assert_eq!(wrapper.execution, Plan);
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[26].facts else {
        panic!("function expected")
    };
    assert!(function.wrapper.complete);
    let PostgresSqlStatementKind::Wrapper { wrapper } = &function.wrapper.statements[0].facts
    else {
        panic!("atomic EXPLAIN expected")
    };
    assert_eq!(wrapper.execution, Runs);
    assert_eq!(result.statements[27].sql, "SELECT 98;");
    for statement in &result.statements {
        assert_eq!(
            statement.sql,
            sql[statement.span.start.offset..statement.span.end.offset]
        );
    }
}
