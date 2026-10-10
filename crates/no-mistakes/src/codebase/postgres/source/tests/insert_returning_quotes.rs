use super::{facts, fixture};
use crate::codebase::postgres::source::*;

fn insert(facts: &PostgresSqlStatementKind) -> &PostgresSqlInsert {
    let PostgresSqlStatementKind::Insert { insert } = facts else {
        panic!("INSERT expected")
    };
    insert
}

#[test]
fn returning_quoted_comment_markers_keep_expression_spans() {
    let sql = fixture("insert-returning-delimiters.sql");
    let result = facts("insert-returning-delimiters.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let quoted = insert(&result.statements[1].facts);
    assert!(quoted.complete, "{:?}", quoted.diagnostics);
    let expected = ["'--'", "'/*x*/'", "\"a--b\""];
    assert_eq!(quoted.returning.len(), expected.len());
    for (item, text) in quoted.returning.iter().zip(expected) {
        let PostgresSqlReturningItem::Expression {
            expression,
            alias: None,
        } = item
        else {
            panic!("expression expected, got {item:?}");
        };
        assert_eq!(expression.sql, text);
        let span = expression.span.as_ref().expect(text);
        assert_eq!(&sql[span.start.offset..span.end.offset], text);
    }
}
