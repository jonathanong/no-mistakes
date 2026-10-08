use super::{facts, fixture};
use crate::codebase::postgres::source::*;

#[test]
fn qualified_end_comment_targets_keep_names_source_spans_and_outer_boundaries() {
    let result = facts("metadata-qualified-end.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 7);
    let sql = fixture("metadata-qualified-end.sql");
    for statement in &result.statements[..3] {
        let PostgresSqlStatementKind::Comment { comment } = &statement.facts else {
            panic!("COMMENT expected")
        };
        assert_eq!(comment.name.parts.last().unwrap().identity, "end");
        assert_eq!(
            statement.sql,
            sql[statement.span.start.offset..statement.span.end.offset]
        );
    }
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[3].facts else {
        panic!("function expected")
    };
    assert!(
        function.wrapper.complete,
        "{:?}",
        function.wrapper.diagnostics
    );
    assert_eq!(function.wrapper.statements.len(), 2);
    for child in &function.wrapper.statements {
        let PostgresSqlStatementKind::Comment { comment } = &child.facts else {
            panic!("nested COMMENT expected")
        };
        assert_eq!(comment.name.parts.last().unwrap().identity, "end");
        assert_eq!(
            child.sql,
            sql[child.span.start.offset..child.span.end.offset]
        );
    }
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[4].facts else {
        panic!("DO expected")
    };
    assert!(block.complete, "{:?}", block.diagnostics);
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!("conditional expected")
    };
    let PostgresSqlStatementKind::Comment { comment } = &branches[0].statements[0].facts else {
        panic!("conditional COMMENT expected")
    };
    assert_eq!(comment.name.sql, "target.end");
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[5].facts else {
        panic!("malformed function expected")
    };
    assert!(!function.wrapper.complete);
    assert_eq!(result.statements[6].sql, "SELECT 99;");
}
