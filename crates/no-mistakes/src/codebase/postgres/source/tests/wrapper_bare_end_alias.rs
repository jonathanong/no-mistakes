use super::{facts, fixture};
use crate::codebase::postgres::source::*;

#[test]
fn bare_end_labels_preserve_atomic_ownership_and_case_closers() {
    let sql = fixture("wrapper-bare-end-alias.sql");
    let result = facts("wrapper-bare-end-alias.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 12);
    for (index, following) in [(0, 2), (2, 3), (4, 4), (6, 0), (8, 5), (10, -1)] {
        let statement = &result.statements[index];
        let PostgresSqlStatementKind::CreateFunction { function } = &statement.facts else {
            panic!("function expected")
        };
        assert!(statement.sql.ends_with("END;"));
        let body = function.body_sql.as_ref().unwrap();
        assert!(body.starts_with("BEGIN ATOMIC"));
        assert!(body.ends_with("END"));
        let span = function.wrapper.span.as_ref().unwrap();
        assert_eq!(span, &statement.span);
        assert_eq!(statement.sql, sql[span.start.offset..span.end.offset]);
        assert_eq!(
            body,
            statement.sql[statement.sql.find("BEGIN ATOMIC").unwrap()..]
                .strip_suffix(';')
                .unwrap()
        );
        if following > 0 {
            assert_eq!(
                function.wrapper.statements.last().unwrap().sql,
                format!("SELECT {following};")
            );
        } else if following == 0 {
            assert!(
                function.wrapper.complete,
                "{:?}",
                function.wrapper.diagnostics
            );
            assert_eq!(function.wrapper.statements.len(), 6);
        } else {
            assert!(!function.wrapper.complete);
            assert!(!function.wrapper.diagnostics.is_empty());
        }
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
    // Native rejection remains explicit; a label must never fake a complete body.
    assert!(!function.wrapper.complete);
    assert!(!function.wrapper.diagnostics.is_empty());
    assert!(function
        .body_sql
        .as_ref()
        .unwrap()
        .contains("end FROM source;"));
    for (index, query) in [(1, 101), (3, 102), (5, 103), (7, 104), (9, 105), (11, 106)] {
        assert_eq!(result.statements[index].sql, format!("SELECT {query};"));
    }
    for statement in &result.statements {
        assert_eq!(
            statement.sql,
            sql[statement.span.start.offset..statement.span.end.offset]
        );
    }
}
