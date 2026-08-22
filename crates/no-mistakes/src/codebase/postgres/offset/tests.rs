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
}
