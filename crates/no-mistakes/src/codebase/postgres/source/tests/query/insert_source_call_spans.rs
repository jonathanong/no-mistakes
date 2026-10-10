use super::*;

#[test]
fn insert_source_select_spans_include_trailing_calls_before_owner_clauses() {
    // sqlparser truncates a bare SELECT source at the function name; the CTE owns its end.
    let sql = fixture("query-insert-source-call-spans.sql");
    let queries = queries("query-insert-source-call-spans.sql");
    assert_eq!(queries.len(), 3);
    for (query, (source_sql, insert_sql, root_sql)) in queries.iter().zip([
        (
            "SELECT now()",
            "INSERT INTO \"té\" SELECT now() RETURNING id",
            "WITH c AS (INSERT INTO \"té\" SELECT now() RETURNING id) SELECT * FROM c",
        ),
        (
            "SELECT lower('é')",
            "INSERT INTO t SELECT lower('é') ON CONFLICT DO NOTHING RETURNING id",
            "WITH c AS (INSERT INTO t SELECT lower('é') ON CONFLICT DO NOTHING RETURNING id) SELECT * FROM c",
        ),
        (
            "SELECT t.returning, t.on, t.conflict, now()",
            "INSERT INTO t SELECT t.returning, t.on, t.conflict, now() ON CONFLICT DO NOTHING RETURNING t.returning",
            "WITH c AS (INSERT INTO t SELECT t.returning, t.on, t.conflict, now() ON CONFLICT DO NOTHING RETURNING t.returning) SELECT * FROM c",
        ),
    ]) {
        assert!(query.complete, "{:?}", query.unsupported);
        assert!(query.unsupported.is_empty());
        assert_eq!(query.nested_statements.len(), 1);
        let child = &query.nested_statements[0];
        assert!(child.complete, "{:?}", child.unsupported);
        assert_eq!(child.sql, insert_sql);
        let child_span = child.span.as_ref().expect("owning INSERT span");
        assert_eq!(&sql[child_span.start.offset..child_span.end.offset], insert_sql);
        let cte_span = query.scopes[child.query_scope_id]
            .span
            .as_ref()
            .expect("owning CTE scope span");
        assert_eq!(&sql[cte_span.start.offset..cte_span.end.offset], insert_sql);
        let PostgresSqlQueryStatementKind::Insert { insert } = &child.facts else {
            panic!("expected typed INSERT source");
        };
        let PostgresSqlCteInsertSource::Select {
            query_scope_id,
            span: Some(span),
        } = &insert.source
        else {
            panic!("expected SELECT source with span");
        };
        assert_eq!(&sql[span.start.offset..span.end.offset], source_sql);
        let scope_span = query.scopes[*query_scope_id]
            .span
            .as_ref()
            .expect("source SELECT scope span");
        assert_eq!(&sql[scope_span.start.offset..scope_span.end.offset], source_sql);
        let root_span = query.scopes[0].span.as_ref().expect("enclosing query span");
        assert_eq!(&sql[root_span.start.offset..root_span.end.offset], root_sql);
    }
}
