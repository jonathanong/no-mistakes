use super::{facts, fixture};
use crate::codebase::postgres::source::*;

#[test]
fn wrappers_recover_at_original_delimiters_and_keep_partial_conflict_facts() {
    let result = facts("wrapper-recovery.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 8);
    let sql = fixture("wrapper-recovery.sql");
    for (ordinal, statement) in result.statements.iter().enumerate() {
        assert_eq!(statement.ordinal, ordinal);
        assert_eq!(
            statement.sql,
            sql[statement.span.start.offset..statement.span.end.offset]
        );
    }
    for index in [0, 2, 6] {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &result.statements[index].facts else {
            panic!("wrapper expected")
        };
        assert!(!wrapper.complete);
        assert_eq!(
            wrapper.execution,
            if index == 2 {
                PostgresSqlExecution::NonExecuting
            } else {
                PostgresSqlExecution::Unknown
            }
        );
        assert!(!wrapper.diagnostics.is_empty());
    }
    for index in [4, 5] {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &result.statements[index].facts else {
            panic!("wrapper expected")
        };
        assert!(wrapper.complete, "{:?}", wrapper.diagnostics);
        let PostgresSqlStatementKind::Insert { insert } = &wrapper.statements[0].facts else {
            panic!("INSERT expected")
        };
        assert_eq!(
            insert
                .on_conflict
                .as_ref()
                .unwrap()
                .predicate
                .as_ref()
                .unwrap()
                .sql,
            "id > 0"
        );
    }
    assert_eq!(result.statements[7].sql, "SELECT 52;");
}

#[test]
fn wrapper_children_reuse_compatibility_restoration_without_changing_facts() {
    let result = facts("wrapper-compatibility.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 7);
    let PostgresSqlStatementKind::Select { query: baseline } = &result.statements[0].facts else {
        panic!("SELECT expected")
    };
    for statement in &result.statements[1..3] {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &statement.facts else {
            panic!("wrapper expected")
        };
        let PostgresSqlStatementKind::Select { query } = &wrapper.statements[0].facts else {
            panic!("SELECT expected")
        };
        assert_eq!(query.columns.len(), baseline.columns.len());
        assert_eq!(query.complete, baseline.complete);
        assert!(wrapper.statements[0].sql.ends_with("ROWS ONLY"));
    }
    let PostgresSqlStatementKind::Wrapper { wrapper } = &result.statements[4].facts else {
        panic!("wrapper expected")
    };
    let PostgresSqlStatementKind::CreateTable { columns, .. } = &wrapper.statements[0].facts else {
        panic!("CREATE TABLE expected")
    };
    let PostgresSqlStatementKind::CreateTable {
        columns: baseline, ..
    } = &result.statements[3].facts
    else {
        panic!("CREATE TABLE expected")
    };
    assert_eq!(
        columns[1].generated.as_ref().unwrap().storage,
        baseline[1].generated.as_ref().unwrap().storage
    );
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[5].facts else {
        panic!("function expected")
    };
    assert!(
        function.wrapper.complete,
        "{:?}",
        function.wrapper.diagnostics
    );
    assert_eq!(function.wrapper.statements.len(), 2);
}

#[test]
fn ctas_wrapper_retains_execution_but_marks_omitted_query_facts_incomplete() {
    let result = facts("wrapper-compatibility.sql");
    let PostgresSqlStatementKind::Wrapper { wrapper } = &result.statements[6].facts else {
        panic!("wrapper expected")
    };
    assert_eq!(wrapper.execution, PostgresSqlExecution::ExecutesForAnalysis);
    assert!(!wrapper.complete);
    assert!(!wrapper.diagnostics.is_empty());
}

#[test]
fn wrapper_boundaries_report_missing_semantics_and_keep_source_ownership() {
    let result = facts("wrapper-boundaries.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 9, "{:?}", result.statements);
    for index in [0, 2] {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &result.statements[index].facts else {
            panic!("wrapper expected")
        };
        assert!(!wrapper.complete);
        assert!(!wrapper.diagnostics.is_empty());
    }
    for index in [4, 6] {
        let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[index].facts
        else {
            panic!("function expected")
        };
        assert!(!function.wrapper.complete);
        assert!(!function.wrapper.diagnostics.is_empty());
    }
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[7].facts else {
        panic!("function expected")
    };
    assert!(
        function.wrapper.complete,
        "{:?}",
        function.wrapper.diagnostics
    );
    assert_eq!(function.wrapper.statements[0].ordinal, 0);
    assert_eq!(function.wrapper.statements[1].ordinal, 1);
    assert_eq!(result.statements[5].sql, "SELECT 56;");
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[8].facts else {
        panic!("DO expected")
    };
    assert!(!block.complete);
    assert!(!block.diagnostics.is_empty());
}
