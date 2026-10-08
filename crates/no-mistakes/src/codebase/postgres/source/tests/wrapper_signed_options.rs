use super::facts;
use crate::codebase::postgres::source::*;

#[test]
fn signed_integer_options_and_transaction_begin_are_bounded() {
    let result = facts("wrapper-signed-options.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 22);
    use PostgresSqlExecution::{ExecutesForAnalysis as Runs, NonExecuting as Plan, Unknown};
    for (statement, expected) in result.statements.iter().zip([
        Runs, Plan, Plan, Unknown, Unknown, Unknown, Unknown, Unknown, Unknown,
    ]) {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &statement.facts else {
            panic!("wrapper expected")
        };
        assert_eq!(wrapper.execution, expected, "{}", statement.sql);
        assert_eq!(wrapper.complete, expected != Unknown);
    }
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[9].facts else {
        panic!("function expected")
    };
    assert!(!function.wrapper.complete);
    assert_eq!(
        function.body_sql.as_deref(),
        Some("BEGIN ATOMIC BEGIN; END")
    );
    assert_eq!(result.statements[10].sql, "SELECT 81;");
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[11].facts else {
        panic!("function expected")
    };
    assert!(function.wrapper.complete);
    assert_eq!(function.wrapper.statements.len(), 1);
    assert_eq!(result.statements[12].sql, "SELECT 82;");
    for (statement, expected) in result.statements[13..].iter().zip([
        Runs, Plan, Runs, Plan, Unknown, Unknown, Unknown, Runs, Plan,
    ]) {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &statement.facts else {
            panic!("wrapper expected")
        };
        assert_eq!(wrapper.execution, expected, "{}", statement.sql);
    }
}
