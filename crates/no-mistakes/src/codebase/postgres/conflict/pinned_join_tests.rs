use super::*;

fn source(sql: &str) -> SqlInsertSourceShape {
    let mut inserts = analyze_conflict_inserts(sql).unwrap();
    assert_eq!(inserts.len(), 1, "{sql}");
    inserts.remove(0).source
}

fn insert(select: &str) -> String {
    format!("INSERT INTO items (id, note) {select} ON CONFLICT (id) DO NOTHING")
}

#[test]
fn pinned_joins_capture_inner_and_cross_join_equalities() {
    let pinned = |select: &str| source(&insert(select)).pinned_source;
    let joined = pinned(
        "SELECT o.id, l.line_no FROM orders o JOIN order_lines l ON l.order_id = o.id WHERE o.id = $1 AND l.line_no = $2",
    )
    .unwrap();
    assert_eq!(joined.relations.len(), 2);
    assert_eq!(joined.relations[0].table, "orders");
    assert_eq!(joined.relations[0].columns, ["id"]);
    assert_eq!(joined.relations[1].table, "order_lines");
    assert_eq!(joined.relations[1].columns, ["line_no"]);
    assert_eq!(
        joined.join_equalities,
        [SqlPinnedJoinEquality {
            left_relation: 1,
            left_column: "order_id".to_string(),
            right_relation: 0,
            right_column: "id".to_string(),
        }]
    );

    let cross = pinned(
        "SELECT o.id, l.line_no FROM orders o CROSS JOIN order_lines l WHERE o.id = $1 AND l.order_id = o.id AND l.line_no = $2",
    )
    .unwrap();
    assert_eq!(cross.join_equalities.len(), 1);

    let comma = pinned(
        "SELECT o.id, l.line_no FROM orders o, order_lines l WHERE o.id = $1 AND l.order_id = o.id AND l.line_no = $2",
    )
    .unwrap();
    assert_eq!(comma.join_equalities.len(), 1);

    let self_join = pinned(
        "SELECT a.id, b.id FROM orders a INNER JOIN orders b ON b.id = a.parent_id WHERE a.id = $1",
    )
    .unwrap();
    assert_eq!(self_join.relations.len(), 2);
    assert_eq!(self_join.join_equalities[0].left_relation, 1);
    assert_eq!(self_join.join_equalities[0].right_relation, 0);
}

#[test]
fn non_inner_unqualified_and_shadowed_joins_stay_unpinned() {
    let none = |select: &str| assert!(source(&insert(select)).pinned_source.is_none(), "{select}");
    none("SELECT id, note FROM orders a, other b WHERE a.id = $1");
    none("SELECT a.id, a.note FROM orders a JOIN other b ON true WHERE a.id = $1");
    none("SELECT o.id, l.line_no FROM orders o LEFT JOIN order_lines l ON l.order_id = o.id WHERE o.id = $1 AND l.line_no = $2");
    none("SELECT o.id, l.line_no FROM orders o RIGHT JOIN order_lines l ON l.order_id = o.id WHERE o.id = $1 AND l.line_no = $2");
    none("SELECT o.id, l.line_no FROM orders o JOIN order_lines l USING (id) WHERE o.id = $1");
    none("SELECT o.id, l.line_no FROM orders o JOIN order_lines l ON l.order_id = o.id OR l.line_no = $2 WHERE o.id = $1");
    none("SELECT o.id, l.line_no FROM orders o JOIN order_lines l ON l.order_id = o.id WHERE id = $1 AND line_no = $2");
    none("SELECT o.id, o.note FROM orders o JOIN orders o ON o.id = o.id WHERE o.id = $1");

    let shadowed_join = analyze_conflict_inserts(
        "WITH order_lines AS (SELECT 1 AS order_id, 1 AS line_no) INSERT INTO items (id, note) SELECT o.id, l.line_no FROM orders o JOIN order_lines l ON l.order_id = o.id WHERE o.id = $1 AND l.line_no = $2 ON CONFLICT (id) DO NOTHING",
    )
    .unwrap();
    assert!(shadowed_join[0].source.pinned_source.is_none());
}
