use super::{facts, fixture};
use crate::codebase::postgres::source::*;

#[test]
fn insert_conflicts_and_into_aliases_preserve_atomic_and_wrapper_boundaries() {
    let result = facts("wrapper-insert-identifier-boundaries.sql");
    let sql = fixture("wrapper-insert-identifier-boundaries.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 8);
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[0].facts else {
        panic!()
    };
    assert!(
        function.wrapper.complete,
        "{:?}",
        function.wrapper.diagnostics
    );
    let mut inserts = vec![&function.wrapper.statements[0]];
    for parent in &result.statements[4..6] {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &parent.facts else {
            panic!()
        };
        assert!(wrapper.complete, "{:?}", wrapper.diagnostics);
        inserts.push(&wrapper.statements[0]);
    }
    for child in inserts {
        let PostgresSqlStatementKind::Insert { insert } = &child.facts else {
            panic!()
        };
        assert!(insert.complete);
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
        let PostgresSqlInsertSource::Select { query, .. } = &insert.source else {
            panic!()
        };
        assert_eq!(query.columns[0].name.parts[0].identity, "begin");
        assert_eq!(
            child.sql,
            sql[child.span.start.offset..child.span.end.offset]
        );
    }
    assert_eq!(result.statements[1].sql, "SELECT 111;");
    assert_eq!(result.statements[3].sql, "SELECT 112;");
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[2].facts else {
        panic!()
    };
    assert!(!function.wrapper.complete);
    assert_eq!(
        function.wrapper.statements[0].sql,
        "SELECT 1 case INTO tmp;"
    );
    assert!(function.wrapper.diagnostics[0]
        .message
        .contains("SELECT INTO"));
    for parent in &result.statements[6..] {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &parent.facts else {
            panic!()
        };
        assert!(!wrapper.complete);
        assert_eq!(wrapper.statements[0].sql, "SELECT 1 case INTO tmp");
    }
}
