use super::{facts, fixture};
use crate::codebase::postgres::source::*;

#[test]
fn atomic_declarations_keep_default_language_first_conflicts_and_nested_ownership() {
    let result = facts("wrapper-review.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 20, "{:?}", result.statements);
    let sql = fixture("wrapper-review.sql");
    for index in [0, 1, 2, 3] {
        let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[index].facts
        else {
            panic!("function expected")
        };
        assert_eq!(
            function.wrapper.complete,
            index < 2,
            "{:?}",
            function.wrapper.diagnostics
        );
        assert_eq!(
            function.wrapper.execution,
            PostgresSqlExecution::NonExecuting
        );
        for child in &function.wrapper.statements {
            assert_eq!(
                child.sql,
                sql[child.span.start.offset..child.span.end.offset]
            );
        }
    }
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[0].facts else {
        unreachable!()
    };
    assert!(function.language.is_none());
    let PostgresSqlStatementKind::Insert { insert } = &function.wrapper.statements[0].facts else {
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
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[1].facts else {
        unreachable!()
    };
    let nested = &function.wrapper.statements[0];
    let PostgresSqlStatementKind::CreateFunction { function: inner } = &nested.facts else {
        panic!("one nested declaration expected")
    };
    assert!(nested.sql.starts_with("CREATE FUNCTION inner_sql"));
    assert!(nested.sql.ends_with("END;"));
    assert_eq!(inner.wrapper.statements.len(), 2);
    assert_eq!(inner.wrapper.statements[1].ordinal, 1);
    assert_eq!(function.wrapper.statements.len(), 2);
    assert_eq!(function.wrapper.statements[1].sql, "SELECT 2;");
    assert_eq!(result.statements[4].sql, "SELECT 58;");
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[18].facts else {
        unreachable!()
    };
    assert!(!function.wrapper.complete);
    assert_eq!(function.wrapper.statements.len(), 2);
    assert_eq!(function.wrapper.statements[0].sql, "SELECT 59;");
    assert_eq!(function.wrapper.statements[0].ordinal, 1);
    assert!(matches!(
        function.wrapper.statements[1].facts,
        PostgresSqlStatementKind::CreateFunction { .. }
    ));
    assert_eq!(result.statements[19].sql, "SELECT 60;");
}

#[test]
fn serialize_and_analyze_dependent_options_have_explicit_execution() {
    let result = facts("wrapper-review.sql");
    for (statement, execution) in result.statements[5..].iter().zip([
        PostgresSqlExecution::ExecutesForAnalysis,
        PostgresSqlExecution::ExecutesForAnalysis,
        PostgresSqlExecution::NonExecuting,
        PostgresSqlExecution::ExecutesForAnalysis,
        PostgresSqlExecution::ExecutesForAnalysis,
        PostgresSqlExecution::Unknown,
        PostgresSqlExecution::Unknown,
        PostgresSqlExecution::Unknown,
        PostgresSqlExecution::Unknown,
        PostgresSqlExecution::NonExecuting,
        PostgresSqlExecution::ExecutesForAnalysis,
        PostgresSqlExecution::Unknown,
        PostgresSqlExecution::Unknown,
    ]) {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &statement.facts else {
            panic!("EXPLAIN expected")
        };
        assert_eq!(wrapper.execution, execution);
        assert_eq!(wrapper.complete, execution != PostgresSqlExecution::Unknown);
    }
}

#[test]
fn wrappers_share_data_modifying_cte_facts_and_original_source_context() {
    let result = facts("wrapper-cte-parity.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 3);
    let sql = fixture("wrapper-cte-parity.sql");
    for statement in &result.statements {
        let wrapper = match &statement.facts {
            PostgresSqlStatementKind::Wrapper { wrapper } => wrapper,
            PostgresSqlStatementKind::CreateFunction { function } => &function.wrapper,
            _ => panic!("wrapper expected"),
        };
        assert!(wrapper.complete, "{:?}", wrapper.diagnostics);
        let PostgresSqlStatementKind::Select { query } = &wrapper.statements[0].facts else {
            panic!("SELECT expected")
        };
        assert_eq!(query.nested_statements.len(), 1);
        let child = &query.nested_statements[0];
        assert!(child.complete);
        let span = child.span.as_ref().unwrap();
        assert_eq!(child.sql, sql[span.start.offset..span.end.offset]);
    }
}

#[test]
fn quoted_languages_conflicting_bodies_and_conditional_wrappers_fail_closed() {
    let result = facts("wrapper-review-boundaries.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 12, "{:?}", result.statements);
    for (index, complete) in [(0, true), (1, false), (2, false), (4, false), (11, false)] {
        let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[index].facts
        else {
            panic!("function expected")
        };
        assert_eq!(
            function.wrapper.complete, complete,
            "{:?}",
            function.wrapper.diagnostics
        );
        if [2, 11].contains(&index) {
            assert_eq!(function.body_sql.as_deref(), Some("AS 'SELECT 1'"));
        }
    }
    assert_eq!(result.statements[3].sql, "SELECT 61;");
    for index in 5..9 {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &result.statements[index].facts else {
            panic!("wrapper expected")
        };
        assert_eq!(wrapper.complete, index < 7);
        assert_eq!(
            wrapper.execution,
            if index < 7 {
                PostgresSqlExecution::NonExecuting
            } else {
                PostgresSqlExecution::Unknown
            }
        );
    }
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[9].facts else {
        panic!("DO expected")
    };
    assert!(block.complete, "{:?}", block.diagnostics);
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!("IF expected")
    };
    let PostgresSqlStatementKind::Wrapper { wrapper } = &branches[0].statements[0].facts else {
        panic!("EXPLAIN expected")
    };
    assert_eq!(wrapper.execution, PostgresSqlExecution::NonExecuting);
    assert_eq!(wrapper.span.as_ref(), Some(&branches[0].statements[0].span));
    let PostgresSqlStatementKind::Conditional { branches: nested } =
        &branches[0].statements[1].facts
    else {
        panic!("nested IF expected")
    };
    let PostgresSqlStatementKind::Wrapper { wrapper } = &nested[0].statements[0].facts else {
        panic!("PREPARE expected")
    };
    assert_eq!(wrapper.execution, PostgresSqlExecution::NonExecuting);
    assert_eq!(wrapper.wrapper_kind, PostgresSqlWrapperKind::Prepare);
    let PostgresSqlStatementKind::Wrapper { wrapper } = &branches[1].statements[0].facts else {
        panic!("ANALYZE expected")
    };
    assert_eq!(wrapper.execution, PostgresSqlExecution::ExecutesForAnalysis);
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[10].facts else {
        panic!("DO expected")
    };
    assert!(!block.complete);
    assert!(!block.diagnostics.is_empty());
}
