use super::facts;
use crate::codebase::postgres::source::*;

#[test]
fn quoted_option_names_and_boolean_nodes_preserve_postgresql_grammar() {
    let result = facts("wrapper-boolean-identity.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 18);
    use PostgresSqlExecution::{ExecutesForAnalysis as Runs, NonExecuting as Plan, Unknown};
    for (statement, expected) in result.statements.iter().zip([
        Runs, Unknown, Unknown, Plan, Unknown, Unknown, Runs, Plan, Runs, Unknown, Runs, Runs,
        Unknown,
    ]) {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &statement.facts else {
            panic!("wrapper expected")
        };
        assert_eq!(wrapper.execution, expected, "{}", statement.sql);
        assert_eq!(wrapper.complete, expected != Unknown, "{}", statement.sql);
    }
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[13].facts else {
        panic!("DO expected")
    };
    assert!(block.complete);
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!("IF expected")
    };
    let PostgresSqlStatementKind::Wrapper { wrapper } = &branches[0].statements[0].facts else {
        panic!("wrapper expected")
    };
    assert_eq!(wrapper.execution, Runs);
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[14].facts else {
        panic!("function expected")
    };
    assert!(function.wrapper.complete);
    let PostgresSqlStatementKind::Wrapper { wrapper } = &function.wrapper.statements[0].facts
    else {
        panic!("wrapper expected")
    };
    assert_eq!(wrapper.execution, Runs);
    assert_eq!(wrapper.statements[0].sql, "SELECT 1;");
}

#[test]
fn legacy_normalization_keeps_non_command_identifier_aliases() {
    let result = facts("wrapper-boolean-identity.sql");
    for statement in &result.statements[15..17] {
        let PostgresSqlStatementKind::Select { query } = &statement.facts else {
            panic!("SELECT expected")
        };
        assert_eq!(query.columns[0].name.parts[0].value, "explain");
    }
    let PostgresSqlStatementKind::Insert { insert } = &result.statements[17].facts else {
        panic!("INSERT expected")
    };
    assert!(insert.complete, "{:?}", insert.diagnostics);
    assert!(matches!(
        &insert.returning[0],
        PostgresSqlReturningItem::Expression { alias: Some(alias), .. } if alias.identity == "analyse"
    ));
}
