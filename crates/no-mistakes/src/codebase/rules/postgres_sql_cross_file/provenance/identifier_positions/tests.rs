use super::*;
use crate::codebase::postgres::EmbeddedSqlCall;

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
