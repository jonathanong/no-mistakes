use super::{facts, fixture};
use crate::codebase::postgres::source::*;

#[test]
fn wrappers_classify_execution_and_preserve_ordered_children() {
    let result = facts("wrappers.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 15);
    let sql = fixture("wrappers.sql");
    for statement in &result.statements {
        assert_eq!(
            statement.sql,
            sql[statement.span.start.offset..statement.span.end.offset]
        );
    }
    for (statement, execution) in result.statements[..10].iter().zip([
        PostgresSqlExecution::NonExecuting,
        PostgresSqlExecution::ExecutesForAnalysis,
        PostgresSqlExecution::NonExecuting,
        PostgresSqlExecution::ExecutesForAnalysis,
        PostgresSqlExecution::ExecutesForAnalysis,
        PostgresSqlExecution::NonExecuting,
        PostgresSqlExecution::ExecutesForAnalysis,
        PostgresSqlExecution::NonExecuting,
        PostgresSqlExecution::ExecutesForAnalysis,
        PostgresSqlExecution::NonExecuting,
    ]) {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &statement.facts else {
            panic!("wrapper expected")
        };
        assert_eq!(wrapper.wrapper_kind, PostgresSqlWrapperKind::Explain);
        assert_eq!(wrapper.execution, execution);
        assert!(wrapper.complete, "{:?}", wrapper.diagnostics);
        assert_eq!(wrapper.statements.len(), 1);
        let child = &wrapper.statements[0];
        assert_eq!(child.ordinal, 0);
        assert_eq!(
            child.sql,
            sql[child.span.start.offset..child.span.end.offset]
        );
        assert_eq!(wrapper.span.as_ref(), Some(&statement.span));
    }
    let PostgresSqlStatementKind::Wrapper { wrapper } = &result.statements[10].facts else {
        panic!("prepare expected")
    };
    assert_eq!(wrapper.wrapper_kind, PostgresSqlWrapperKind::Prepare);
    assert_eq!(wrapper.execution, PostgresSqlExecution::NonExecuting);
    assert!(wrapper.complete);
    assert!(wrapper.statements[0].sql.trim_start().starts_with("INSERT"));
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[11].facts else {
        panic!("function expected")
    };
    assert_eq!(function.name.sql, "f");
    assert_eq!(
        function.wrapper.execution,
        PostgresSqlExecution::NonExecuting
    );
    assert!(
        function.wrapper.complete,
        "{:?}",
        function.wrapper.diagnostics
    );
    assert_eq!(function.wrapper.statements.len(), 2);
    assert!(matches!(
        function.wrapper.statements[0].facts,
        PostgresSqlStatementKind::Insert { .. }
    ));
    assert!(function
        .body_sql
        .as_ref()
        .unwrap()
        .starts_with("BEGIN ATOMIC"));
    assert!(function.body_sql.as_ref().unwrap().ends_with("END"));
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[13].facts else {
        panic!("opaque function expected")
    };
    assert!(!function.wrapper.complete);
    assert_eq!(
        function.wrapper.execution,
        PostgresSqlExecution::NonExecuting
    );
    assert!(function.wrapper.statements.is_empty());
    assert_eq!(function.wrapper.diagnostics.len(), 1);
}

#[test]
fn unsupported_options_fail_closed_without_losing_neighboring_statements() {
    let result = facts("wrapper-options.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 29);
    for (index, statement) in result.statements.iter().enumerate() {
        if matches!(index, 21 | 23) {
            assert!(matches!(
                statement.facts,
                PostgresSqlStatementKind::Select { .. }
            ));
            continue;
        }
        let PostgresSqlStatementKind::Wrapper { wrapper } = &statement.facts else {
            panic!("wrapper {index} expected")
        };
        let known = matches!(index, 0 | 1 | 9 | 16 | 19 | 28);
        assert_eq!(
            wrapper.complete, known,
            "index {index}: {:?}",
            wrapper.diagnostics
        );
        if index >= 24 {
            for child in &wrapper.statements {
                assert_eq!(child.sql, "SELECT 1");
            }
        }
        if !known {
            assert!(!wrapper.diagnostics.is_empty());
        }
    }
}

#[test]
fn atomic_function_recovery_preserves_authoritative_end_and_neighbors() {
    let result = facts("wrapper-functions.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 12, "{:?}", result.statements);
    for statement in &result.statements {
        match &statement.facts {
            PostgresSqlStatementKind::CreateFunction { function } => {
                assert_eq!(
                    function.wrapper.execution,
                    PostgresSqlExecution::NonExecuting
                );
                if function.name.sql == "empty_body" {
                    assert!(function.wrapper.complete);
                } else {
                    assert!(!function.wrapper.complete);
                    assert!(!function.wrapper.diagnostics.is_empty());
                }
            }
            PostgresSqlStatementKind::Select { .. } => assert!(statement.sql.starts_with("SELECT")),
            _ => panic!("no END fragments permitted"),
        }
    }
}
