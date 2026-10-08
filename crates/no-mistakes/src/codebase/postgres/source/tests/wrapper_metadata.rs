use super::{facts, fixture};
use crate::codebase::postgres::source::*;

#[test]
fn wrapper_metadata_uses_prepared_validation_spans_and_authoritative_boundaries() {
    let result = facts("wrapper-metadata.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 10, "{:?}", result.statements);
    let sql = fixture("wrapper-metadata.sql");
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[0].facts else {
        panic!("function expected")
    };
    assert!(
        function.wrapper.complete,
        "{:?}",
        function.wrapper.diagnostics
    );
    assert_eq!(function.wrapper.statements.len(), 4);
    for (child, expected) in
        function.wrapper.statements[..3]
            .iter()
            .zip([Some("café"), Some("first second"), None])
    {
        let PostgresSqlStatementKind::Comment { comment } = &child.facts else {
            panic!("COMMENT expected")
        };
        assert_eq!(comment.comment.as_deref(), expected);
        assert_eq!(
            child.sql,
            sql[child.span.start.offset..child.span.end.offset]
        );
        assert!(child.sql.ends_with(';'));
    }
    for index in [1, 7, 9] {
        let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[index].facts
        else {
            panic!("function expected")
        };
        assert!(!function.wrapper.complete);
        assert!(!function.wrapper.diagnostics.is_empty());
    }
    assert_eq!(result.statements[2].sql, "SELECT 62;");
    assert_eq!(result.statements[8].sql, "SELECT 63;");
    for index in [3, 4, 5] {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &result.statements[index].facts else {
            panic!("wrapper expected")
        };
        assert!(!wrapper.complete);
        assert_eq!(
            wrapper.execution,
            if index == 4 {
                PostgresSqlExecution::NonExecuting
            } else {
                PostgresSqlExecution::Unknown
            }
        );
        if index < 5 {
            let child = &wrapper.statements[0];
            assert!(matches!(
                child.facts,
                PostgresSqlStatementKind::Comment { .. }
            ));
            assert_eq!(
                child.sql,
                sql[child.span.start.offset..child.span.end.offset]
            );
            assert!(
                child.sql.ends_with("'not executable'") || child.sql.ends_with("'not preparable'")
            );
        } else {
            assert!(wrapper.statements.is_empty());
        }
    }
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[6].facts else {
        panic!("DO expected")
    };
    assert!(!block.complete);
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!("IF expected")
    };
    assert!(matches!(
        branches[0].statements[0].facts,
        PostgresSqlStatementKind::Comment { .. }
    ));
}

#[test]
fn prepared_comment_at_eof_keeps_its_full_original_source_span() {
    for name in [
        "wrapper-comment-eof.sql",
        "wrapper-routine-comment-eof.sql",
        "wrapper-explain-comment-eof.sql",
        "wrapper-prepare-comment-eof.sql",
    ] {
        let result = facts(name);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert_eq!(result.statements.len(), 1);
        let sql = fixture(name);
        let parent = &result.statements[0];
        assert_eq!(parent.sql, sql.trim_end());
        if let PostgresSqlStatementKind::Wrapper { wrapper } = &parent.facts {
            assert!(!wrapper.complete);
            assert_eq!(wrapper.span.as_ref(), Some(&parent.span));
            let child = &wrapper.statements[0];
            assert_eq!(
                child.sql,
                sql[child.span.start.offset..child.span.end.offset]
            );
            assert!(child.sql.ends_with("at EOF'"));
        } else {
            assert!(matches!(
                parent.facts,
                PostgresSqlStatementKind::Comment { .. }
            ));
        }
    }
}
