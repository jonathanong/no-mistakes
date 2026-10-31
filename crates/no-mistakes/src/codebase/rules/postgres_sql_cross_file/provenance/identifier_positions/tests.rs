use super::*;

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
