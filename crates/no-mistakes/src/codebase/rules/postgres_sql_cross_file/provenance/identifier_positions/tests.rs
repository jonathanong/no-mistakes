use super::*;
use crate::codebase::postgres::EmbeddedSqlCall;
use sqlparser::ast::{GroupByExpr, SetExpr, Statement, UpdateTableFromKind};

fn collected(sql: &str) -> (BTreeSet<u32>, BTreeSet<u32>) {
    let statements = parse_postgres_sql(sql).expect("sql");
    let found = indexes(&statements);
    (found.relations, found.columns)
}

#[test]
fn bind_positions_are_not_identifier_indexes() {
    let (relations, columns) = collected(
        "SELECT sql_placeholder_1 FROM sql_placeholder_2 WHERE user_id = sql_placeholder_3 AND id = ANY(sql_placeholder_4) LIMIT sql_placeholder_5",
    );
    assert_eq!(columns, BTreeSet::from([1]));
    assert_eq!(relations, BTreeSet::from([2]));
}

#[test]
fn order_group_update_and_insert_columns_are_identifier_indexes() {
    let (relations, columns) =
        collected("SELECT id FROM items GROUP BY sql_placeholder_1 ORDER BY sql_placeholder_2");
    assert!(relations.is_empty());
    assert_eq!(columns, BTreeSet::from([1, 2]));
    let (relations, columns) =
        collected("UPDATE sql_placeholder_1 SET sql_placeholder_2 = sql_placeholder_3");
    assert_eq!(relations, BTreeSet::from([1]));
    assert_eq!(columns, BTreeSet::from([2]));
    let (relations, columns) =
        collected("INSERT INTO sql_placeholder_1 (sql_placeholder_2) VALUES (sql_placeholder_3)");
    assert_eq!(relations, BTreeSet::from([1]));
    assert_eq!(columns, BTreeSet::from([2]));
}

#[test]
fn other_statement_shapes_keep_identifier_indexes() {
    let (relations, columns) = collected(
        "WITH c AS (SELECT sql_placeholder_1) SELECT sql_placeholder_2 FROM c JOIN sql_placeholder_3 ON true",
    );
    assert_eq!(columns, BTreeSet::from([1, 2]));
    assert_eq!(relations, BTreeSet::from([3]));
    let (relations, columns) = collected(
        "SELECT (sql_placeholder_1), t.sql_placeholder_2 FROM (SELECT sql_placeholder_3) AS d",
    );
    assert_eq!(columns, BTreeSet::from([1, 2, 3]));
    assert!(relations.is_empty());
    let (relations, columns) =
        collected("SELECT sql_placeholder_1 UNION ALL SELECT sql_placeholder_2");
    assert_eq!(columns, BTreeSet::from([1, 2]));
    assert!(relations.is_empty());
    let (relations, columns) = collected("INSERT INTO sql_placeholder_1 SELECT sql_placeholder_2");
    assert_eq!(relations, BTreeSet::from([1]));
    assert_eq!(columns, BTreeSet::from([2]));
    let (relations, columns) = collected(
        "UPDATE sql_placeholder_1 SET (sql_placeholder_2, sql_placeholder_3) = (1, 2) FROM sql_placeholder_4",
    );
    assert_eq!(relations, BTreeSet::from([1, 4]));
    assert_eq!(columns, BTreeSet::from([2, 3]));
    let (relations, columns) = collected("DELETE FROM sql_placeholder_1 USING sql_placeholder_2");
    assert_eq!(relations, BTreeSet::from([1, 2]));
    assert!(columns.is_empty());
    let (relations, columns) = collected(
        "SELECT sql_placeholder_1 FROM (sql_placeholder_2 JOIN sql_placeholder_3 ON true) AS j",
    );
    assert_eq!(columns, BTreeSet::from([1]));
    assert_eq!(relations, BTreeSet::from([2, 3]));
    let (relations, columns) = collected("SELECT sql_placeholder_0, sql_placeholder_");
    assert!(relations.is_empty());
    assert!(columns.is_empty());
}

#[test]
fn nested_dml_and_non_select_shapes_keep_identifier_indexes() {
    let (relations, columns) =
        collected("UPDATE sql_placeholder_1 FROM sql_placeholder_2 SET sql_placeholder_3 = 1");
    assert_eq!(relations, BTreeSet::from([1, 2]));
    assert_eq!(columns, BTreeSet::from([3]));
    let (relations, columns) = collected("SELECT sql_placeholder_1 GROUP BY ALL");
    assert!(relations.is_empty());
    assert_eq!(columns, BTreeSet::from([1]));
    let (relations, columns) =
        collected("SELECT sql_placeholder_1 FROM ((SELECT sql_placeholder_2)) AS nested");
    assert!(relations.is_empty());
    assert_eq!(columns, BTreeSet::from([1, 2]));
    let (relations, columns) = collected(
        "WITH c AS (SELECT sql_placeholder_1) INSERT INTO sql_placeholder_2 SELECT sql_placeholder_3",
    );
    assert_eq!(relations, BTreeSet::from([2]));
    assert_eq!(columns, BTreeSet::from([1, 3]));
    let (relations, columns) = collected("EXPLAIN SELECT 1");
    assert!(relations.is_empty());
    assert!(columns.is_empty());
    let update = parse_postgres_sql(
        "UPDATE sql_placeholder_1 FROM sql_placeholder_2 SET sql_placeholder_3 = 1",
    )
    .expect("sql");
    assert!(matches!(
        &update[0],
        Statement::Update(update) if matches!(update.from, Some(UpdateTableFromKind::BeforeSet(_)))
    ));
    let grouped = parse_postgres_sql("SELECT sql_placeholder_1 GROUP BY ALL").expect("sql");
    let Statement::Query(query) = &grouped[0] else {
        panic!("query");
    };
    let SetExpr::Select(select) = query.body.as_ref() else {
        panic!("select");
    };
    assert!(matches!(select.group_by, GroupByExpr::All(_)));
    let nested =
        parse_postgres_sql("SELECT sql_placeholder_1 FROM ((SELECT sql_placeholder_2)) AS nested")
            .expect("sql");
    let Statement::Query(query) = &nested[0] else {
        panic!("query");
    };
    let SetExpr::Select(select) = query.body.as_ref() else {
        panic!("select");
    };
    let sqlparser::ast::TableFactor::Derived { subquery, .. } = &select.from[0].relation else {
        panic!("derived");
    };
    assert!(matches!(subquery.body.as_ref(), SetExpr::Query(_)));
    let inserted = parse_postgres_sql(
        "WITH c AS (SELECT sql_placeholder_1) INSERT INTO sql_placeholder_2 SELECT sql_placeholder_3",
    )
    .expect("sql");
    let Statement::Query(query) = &inserted[0] else {
        panic!("query");
    };
    assert!(matches!(query.body.as_ref(), SetExpr::Insert(_)));
    let explained = parse_postgres_sql("EXPLAIN SELECT 1").expect("sql");
    assert!(!matches!(
        &explained[0],
        Statement::Query(_) | Statement::Insert(_) | Statement::Update(_) | Statement::Delete(_)
    ));
}

#[test]
fn column_only_text_scans_and_unparsed_text_does_not_align() {
    let mut call = EmbeddedSqlCall {
        sql_text: Some("SELECT sql_placeholder_1 FROM users".to_string()),
        ..EmbeddedSqlCall::default()
    };
    assert!(scans_identifier(&call));
    assert_eq!(aligned(&call), Some((BTreeSet::from([1]), 1)));
    call.sql_text = Some("SELECT 1".to_string());
    assert!(!scans_identifier(&call));
    assert!(aligned(&call).is_none());
    call.sql_text = None;
    assert!(!scans_identifier(&call));
    assert!(aligned(&call).is_none());
    call.sql_text = Some("%%%".to_string());
    assert!(aligned(&call).is_none());
}

#[test]
fn glued_prefix_and_suffix_still_mark_the_hole() {
    let call = EmbeddedSqlCall {
        sql_text: Some(
            "SELECT id FROM sql_placeholder_1__votes JOIN audit_sql_placeholder_2 ON true JOIN sql_placeholder_3_sql_placeholder_4 ON true"
                .to_string(),
        ),
        ..EmbeddedSqlCall::default()
    };
    assert_eq!(aligned(&call), Some((BTreeSet::from([1, 2, 3, 4]), 4)));
}

#[test]
fn function_call_arguments_are_not_relation_names() {
    for sql in [
        "SELECT t FROM UNNEST(sql_placeholder_1::uuid[]) AS input(t)",
        "SELECT c.k FROM unnest(sql_placeholder_1::text[], sql_placeholder_2::uuid[]) WITH ORDINALITY AS c (k, i)",
        "SELECT g FROM generate_series(1, sql_placeholder_1) g",
        "WITH p AS (SELECT id FROM posts WHERE id = ANY(sql_placeholder_1::uuid[])) SELECT id FROM p",
    ] {
        let (relations, columns) = collected(sql);
        assert!(relations.is_empty(), "{sql}");
        assert!(columns.is_empty(), "{sql}");
    }
    let (relations, _) = collected(
        "SELECT 1 FROM sql_placeholder_1 JOIN unnest(sql_placeholder_2::uuid[]) AS u(id) ON true JOIN schema.sql_placeholder_3 ON true",
    );
    assert_eq!(relations, BTreeSet::from([1, 3]));
}
