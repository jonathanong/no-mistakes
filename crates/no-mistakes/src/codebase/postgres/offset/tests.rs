use super::sql_has_offset_clause;

#[test]
fn offset_keyword_is_detected() {
    assert!(sql_has_offset_clause("SELECT id FROM posts OFFSET 10").unwrap());
    assert!(
        sql_has_offset_clause("SELECT id FROM posts LIMIT 10 OFFSET sql_placeholder_1").unwrap()
    );
}

#[test]
fn limit_without_offset_is_clean() {
    assert!(!sql_has_offset_clause("SELECT id FROM posts ORDER BY id DESC LIMIT 11").unwrap());
}

#[test]
fn offset_in_string_literal_is_ignored() {
    assert!(!sql_has_offset_clause(
        "INSERT INTO examples (body) VALUES ('offset by a travel credit')"
    )
    .unwrap());
}

#[test]
fn subquery_and_cte_offsets_are_detected() {
    assert!(
        sql_has_offset_clause("SELECT * FROM (SELECT id FROM posts OFFSET 1) AS page").unwrap()
    );
    assert!(sql_has_offset_clause(
        "WITH page AS (SELECT id FROM posts OFFSET 1) SELECT * FROM page"
    )
    .unwrap());
}

#[test]
fn insert_select_offset_is_detected() {
    assert!(sql_has_offset_clause("INSERT INTO t SELECT * FROM u OFFSET 5").unwrap());
}

#[test]
fn unparseable_sql_returns_error() {
    let error = sql_has_offset_clause("SELECT id FROM posts OFFSET").expect_err("unparseable");
    assert!(!error.message.is_empty());
}

#[test]
fn non_query_statements_are_clean() {
    assert!(!sql_has_offset_clause("CREATE TABLE t (id int)").unwrap());
    assert!(!sql_has_offset_clause("DROP TABLE t").unwrap());
}

#[test]
fn union_offset_is_detected() {
    assert!(sql_has_offset_clause("SELECT id FROM a UNION SELECT id FROM b OFFSET 2").unwrap());
}

#[test]
fn in_subquery_offset_is_detected() {
    assert!(sql_has_offset_clause(
        "SELECT id FROM posts WHERE id IN (SELECT id FROM other OFFSET 1)"
    )
    .unwrap());
}

#[test]
fn join_derived_offset_is_detected() {
    assert!(sql_has_offset_clause(
        "SELECT * FROM t JOIN (SELECT id FROM u OFFSET 1) AS page ON true"
    )
    .unwrap());
}

#[test]
fn parenthesized_predicate_without_offset_is_clean() {
    assert!(!sql_has_offset_clause("SELECT * FROM t WHERE (id = ANY($1))").unwrap());
}

#[test]
fn union_arm_offset_is_detected() {
    assert!(sql_has_offset_clause(
        "SELECT id FROM a WHERE id IN (SELECT x FROM t OFFSET 1) UNION SELECT id FROM b"
    )
    .unwrap());
}

#[test]
fn parenthesized_union_offset_is_detected() {
    assert!(
        sql_has_offset_clause("(SELECT id FROM posts OFFSET 1) UNION SELECT id FROM other")
            .unwrap()
    );
}

#[test]
fn scalar_subquery_offset_is_detected() {
    assert!(sql_has_offset_clause(
        "SELECT id FROM posts WHERE id = (SELECT id FROM other OFFSET 1)"
    )
    .unwrap());
}

#[test]
fn binary_and_unary_predicate_offsets_are_detected() {
    assert!(sql_has_offset_clause(
        "SELECT id FROM posts WHERE live AND id IN (SELECT id FROM other OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT id FROM posts WHERE NOT (id IN (SELECT id FROM other OFFSET 1))"
    )
    .unwrap());
}

#[test]
fn exists_subquery_offset_is_detected() {
    assert!(
        sql_has_offset_clause("SELECT * FROM t WHERE EXISTS (SELECT 1 FROM u OFFSET 1)").unwrap()
    );
}

#[test]
fn projection_subquery_offset_is_detected() {
    assert!(sql_has_offset_clause("SELECT (SELECT 1 FROM u OFFSET 1) FROM t").unwrap());
}

#[test]
fn join_on_exists_offset_is_detected() {
    assert!(
        sql_has_offset_clause("SELECT * FROM t JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)")
            .unwrap()
    );
    assert!(sql_has_offset_clause(
        "SELECT * FROM t LEFT JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT * FROM t RIGHT JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT * FROM t FULL JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT * FROM t INNER JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT * FROM t LEFT OUTER JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT * FROM t RIGHT OUTER JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)"
    )
    .unwrap());
}

#[test]
fn order_by_subquery_offset_is_detected() {
    assert!(
        sql_has_offset_clause("SELECT id FROM t ORDER BY (SELECT id FROM u OFFSET 1 LIMIT 1)")
            .unwrap()
    );
    assert!(!sql_has_offset_clause("SELECT id FROM t ORDER BY id").unwrap());
    assert!(!sql_has_offset_clause("SELECT id FROM t ORDER BY ALL").unwrap());
}

#[test]
fn create_table_and_view_offsets_are_detected() {
    assert!(sql_has_offset_clause("CREATE TABLE page AS SELECT * FROM posts OFFSET 10").unwrap());
    assert!(sql_has_offset_clause("CREATE VIEW page AS SELECT * FROM posts OFFSET 10").unwrap());
}

#[test]
fn having_exists_offset_is_detected() {
    assert!(sql_has_offset_clause(
        "SELECT id FROM t GROUP BY id HAVING EXISTS (SELECT 1 FROM u OFFSET 1)"
    )
    .unwrap());
}

#[test]
fn join_without_on_subquery_is_clean() {
    assert!(!sql_has_offset_clause("SELECT * FROM t JOIN u USING (id)").unwrap());
    assert!(!sql_has_offset_clause("SELECT * FROM t CROSS JOIN u").unwrap());
}

#[test]
fn update_and_delete_subquery_offsets_are_detected() {
    assert!(
        sql_has_offset_clause("UPDATE users SET rank = (SELECT rank FROM rankings OFFSET 1)")
            .unwrap()
    );
    assert!(
        sql_has_offset_clause("DELETE FROM users WHERE id IN (SELECT id FROM stale OFFSET 1)")
            .unwrap()
    );
    assert!(sql_has_offset_clause(
        "UPDATE users SET rank = 1 WHERE id IN (SELECT id FROM stale OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "UPDATE users SET rank = 1 FROM (SELECT id FROM stale OFFSET 1) AS s WHERE users.id = s.id"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "DELETE FROM users USING (SELECT id FROM stale OFFSET 1) AS s WHERE users.id = s.id"
    )
    .unwrap());
}

#[test]
fn values_row_subquery_offset_is_detected() {
    assert!(sql_has_offset_clause(
        "INSERT INTO t(id) VALUES ((SELECT id FROM u OFFSET 1 LIMIT 1))"
    )
    .unwrap());
    assert!(sql_has_offset_clause("VALUES ((SELECT id FROM u OFFSET 1 LIMIT 1))").unwrap());
}

#[test]
fn explain_analyze_offset_is_detected() {
    assert!(sql_has_offset_clause("EXPLAIN ANALYZE SELECT id FROM posts OFFSET 10").unwrap());
    assert!(!sql_has_offset_clause("EXPLAIN SELECT id FROM posts LIMIT 10").unwrap());
}

#[test]
fn returning_and_on_conflict_offsets_are_detected() {
    assert!(sql_has_offset_clause(
        "INSERT INTO audit DEFAULT VALUES RETURNING (SELECT id FROM pages OFFSET 1 LIMIT 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "UPDATE users SET rank = 1 RETURNING (SELECT id FROM pages OFFSET 1 LIMIT 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "DELETE FROM users RETURNING (SELECT id FROM pages OFFSET 1 LIMIT 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "INSERT INTO t(id, value) VALUES (1, 1) ON CONFLICT (id) DO UPDATE SET value = (SELECT value FROM u OFFSET 1 LIMIT 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "INSERT INTO t(id) VALUES (1) ON CONFLICT (id) DO UPDATE SET id = 1 WHERE id IN (SELECT id FROM u OFFSET 1)"
    )
    .unwrap());
}

#[test]
fn nested_join_group_by_distinct_and_limit_offsets_are_detected() {
    assert!(sql_has_offset_clause(
        "SELECT * FROM (a JOIN (SELECT * FROM b OFFSET 1) AS page ON true) AS joined"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT count(*) FROM t GROUP BY (SELECT id FROM pages OFFSET 1 LIMIT 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT DISTINCT ON ((SELECT id FROM pages OFFSET 1 LIMIT 1)) id FROM t"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT * FROM t LIMIT (SELECT id FROM limits OFFSET 1 LIMIT 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT count(*) OVER w FROM t WINDOW w AS (ORDER BY (SELECT id FROM u OFFSET 1 LIMIT 1))"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT count(*) OVER w FROM t WINDOW w AS (PARTITION BY (SELECT id FROM u OFFSET 1 LIMIT 1))"
    )
    .unwrap());
}

#[test]
fn function_args_and_modifying_cte_offsets_are_detected() {
    assert!(
        sql_has_offset_clause("SELECT COALESCE((SELECT id FROM pages OFFSET 1 LIMIT 1), 0)")
            .unwrap()
    );
    assert!(!sql_has_offset_clause("SELECT COUNT(*) FROM t").unwrap());
    assert!(sql_has_offset_clause(
        "WITH changed AS (UPDATE users SET rank = (SELECT rank FROM rankings OFFSET 1) RETURNING id) SELECT * FROM changed"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "WITH removed AS (DELETE FROM users WHERE id IN (SELECT id FROM stale OFFSET 1) RETURNING id) SELECT * FROM removed"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "WITH added AS (INSERT INTO t SELECT id FROM u OFFSET 1 RETURNING id) SELECT * FROM added"
    )
    .unwrap());
    assert!(!sql_has_offset_clause("SELECT CURRENT_TIMESTAMP").unwrap());
}
