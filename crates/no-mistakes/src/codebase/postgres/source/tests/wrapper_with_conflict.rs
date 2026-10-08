use super::{facts, fixture};
use crate::codebase::postgres::source::*;

fn insert(facts: &PostgresSqlStatementKind) -> &PostgresSqlInsert {
    let PostgresSqlStatementKind::Insert { insert } = facts else {
        panic!("INSERT expected")
    };
    insert
}

#[test]
fn wrapped_with_inserts_share_partial_conflict_and_source_provenance() {
    let result = facts("wrapper-with-partial-conflict.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 9, "{:?}", result.statements);
    let sql = fixture("wrapper-with-partial-conflict.sql");
    let baseline = insert(&result.statements[0].facts);
    assert!(baseline.complete, "{:?}", baseline.diagnostics);
    let mut children = Vec::new();
    for parent in &result.statements[1..3] {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &parent.facts else {
            panic!()
        };
        assert!(wrapper.complete, "{:?}", wrapper.diagnostics);
        children.push(&wrapper.statements[0]);
    }
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[3].facts else {
        panic!()
    };
    assert!(
        function.wrapper.complete,
        "{:?}",
        function.wrapper.diagnostics
    );
    children.push(&function.wrapper.statements[0]);
    for child in children {
        assert_eq!(
            child.sql,
            sql[child.span.start.offset..child.span.end.offset]
        );
        assert!(child.sql.starts_with("WITH src"));
        let insert = insert(&child.facts);
        assert!(insert.complete);
        let conflict = insert.on_conflict.as_ref().unwrap();
        let baseline_conflict = baseline.on_conflict.as_ref().unwrap();
        assert_eq!(conflict.target, baseline_conflict.target);
        assert_eq!(conflict.action, baseline_conflict.action);
        assert_eq!(conflict.predicate.as_ref().unwrap().sql, "id > 0");
        let predicate_span = conflict.predicate.as_ref().unwrap().span.as_ref().unwrap();
        assert_eq!(
            &sql[predicate_span.start.offset..predicate_span.end.offset],
            "id > 0"
        );
        let PostgresSqlInsertSource::Select { query, span } = &insert.source else {
            panic!()
        };
        let PostgresSqlInsertSource::Select {
            query: baseline_query,
            ..
        } = &baseline.source
        else {
            panic!()
        };
        assert_eq!(query.ctes.len(), baseline_query.ctes.len());
        assert_eq!(query.ctes[0].name, baseline_query.ctes[0].name);
        assert_eq!(query.ctes[0].referenced, baseline_query.ctes[0].referenced);
        assert_eq!(query.ctes[0].used, baseline_query.ctes[0].used);
        assert!(query.ctes[0].referenced && query.ctes[0].used);
        let span = span.as_ref().unwrap();
        assert_eq!(
            &sql[span.start.offset..span.end.offset],
            "SELECT id FROM src"
        );
    }
    let PostgresSqlStatementKind::Wrapper { wrapper } = &result.statements[4].facts else {
        panic!()
    };
    assert!(!wrapper.complete);
    assert!(!insert(&wrapper.statements[0].facts).complete);
    for index in [5, 7] {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &result.statements[index].facts else {
            panic!()
        };
        assert!(!wrapper.complete);
        assert!(!wrapper.diagnostics.is_empty());
    }
    assert_eq!(result.statements[6].sql, "SELECT 71;");
    assert_eq!(result.statements[8].sql, "SELECT 72;");
}
