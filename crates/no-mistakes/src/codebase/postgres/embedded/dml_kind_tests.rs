use super::dml_kind::{
    recovered_sql_may_select, recovered_sql_may_write_columns, recovered_sql_needs_insert_check,
    top_level_dml_kind, TopLevelDml,
};

#[test]
fn classifies_plain_dml_keywords() {
    assert_eq!(top_level_dml_kind("SELECT 1"), Some(TopLevelDml::Select));
    assert_eq!(
        top_level_dml_kind("INSERT INTO items VALUES (1)"),
        Some(TopLevelDml::Insert)
    );
    assert_eq!(
        top_level_dml_kind("UPDATE items SET gone = true"),
        Some(TopLevelDml::Update)
    );
    assert_eq!(
        top_level_dml_kind("DELETE FROM items"),
        Some(TopLevelDml::Delete)
    );
    assert_eq!(
        top_level_dml_kind("MERGE INTO items USING src ON true WHEN MATCHED THEN DELETE"),
        Some(TopLevelDml::Merge)
    );
}

#[test]
fn skips_complete_with_prefix_to_the_final_statement() {
    let sql = "WITH stale AS (SELECT id FROM items WHERE expired) UPDATE items SET gone = true FROM stale WHERE items.id = stale.id";
    assert_eq!(top_level_dml_kind(sql), Some(TopLevelDml::Update));
    assert!(!recovered_sql_needs_insert_check(Some(sql)));
}

#[test]
fn skips_insert_inside_a_cte_when_the_final_statement_is_update() {
    let sql = "WITH logged AS (INSERT INTO log SELECT id FROM items) UPDATE items SET gone = true";
    assert_eq!(top_level_dml_kind(sql), Some(TopLevelDml::Update));
    assert!(!recovered_sql_needs_insert_check(Some(sql)));
}

#[test]
fn recovers_with_insert_as_insert_family() {
    let sql = "WITH input AS (SELECT 1 AS id) INSERT INTO items (id) SELECT id FROM input";
    assert_eq!(top_level_dml_kind(sql), Some(TopLevelDml::Insert));
    assert!(recovered_sql_needs_insert_check(Some(sql)));
}

#[test]
fn incomplete_with_prefix_fails_closed() {
    let sql = "WITH stale AS (SELECT id FROM items";
    assert_eq!(top_level_dml_kind(sql), None);
    assert!(recovered_sql_needs_insert_check(Some(sql)));
}

#[test]
fn missing_sql_needs_insert_check() {
    assert!(recovered_sql_needs_insert_check(None));
}

#[test]
fn skips_comments_and_recursive_materialized_ctes() {
    let sql = "WITH RECURSIVE tree AS NOT MATERIALIZED (SELECT 1) -- hi\nSELECT 1";
    assert_eq!(top_level_dml_kind(sql), Some(TopLevelDml::Select));
}

#[test]
fn skips_comma_separated_ctes_and_quoted_names() {
    let sql = r#"WITH "stale" AS (SELECT 1), other AS (SELECT 2) DELETE FROM items"#;
    assert_eq!(top_level_dml_kind(sql), Some(TopLevelDml::Delete));
}

fn assert_final_update(sql: &str) {
    assert_eq!(top_level_dml_kind(sql), Some(TopLevelDml::Update), "{sql}");
}

#[test]
fn skips_delimited_text_inside_cte_bodies() {
    assert_final_update("WITH stale AS (SELECT ')') UPDATE items SET x = 1");
    assert_final_update(r#"WITH stale AS (SELECT ")") UPDATE items SET x = 1"#);
    assert_final_update("WITH stale AS (SELECT 'it''s') UPDATE items SET x = 1");
    assert_final_update(r#"WITH stale AS (SELECT "a""b") UPDATE items SET x = 1"#);
    assert_final_update(r#"WITH stale AS (SELECT 'a\'b') UPDATE items SET x = 1"#);
    assert_final_update("WITH stale AS (SELECT $$)$$) UPDATE items SET x = 1");
    assert_final_update("WITH stale AS (SELECT $body$) $body$) UPDATE items SET x = 1");
    assert_final_update("WITH stale AS (SELECT $_x$) $_x$) UPDATE items SET x = 1");
    assert_final_update("WITH stale AS (SELECT $1) UPDATE items SET x = 1");
    assert_final_update("WITH stale AS (SELECT $tag) UPDATE items SET x = 1");
    assert_final_update("WITH stale AS (SELECT -- )\n1) UPDATE items SET x = 1");
    assert_final_update("WITH stale AS (SELECT /* ) */ 1) UPDATE items SET x = 1");
}

#[test]
fn unterminated_delimiters_inside_ctes_fail_closed() {
    assert_eq!(top_level_dml_kind("WITH stale AS (SELECT 'nope"), None);
    assert_eq!(top_level_dml_kind(r#"WITH stale AS (SELECT "nope"#), None);
    assert_eq!(top_level_dml_kind("WITH stale AS (SELECT $body$nope"), None);
    assert_eq!(top_level_dml_kind("WITH stale AS (SELECT $"), None);
}

#[test]
fn skip_cte_column_lists_materialized_and_malformed_prefixes() {
    assert_eq!(
        top_level_dml_kind("WITH cte(a, b) AS (SELECT 1) SELECT 1"),
        Some(TopLevelDml::Select)
    );
    assert_eq!(
        top_level_dml_kind("WITH cte AS MATERIALIZED (SELECT 1) SELECT 1"),
        Some(TopLevelDml::Select)
    );
    assert_eq!(top_level_dml_kind("WITH AS (SELECT 1) SELECT 1"), None);
    assert_eq!(top_level_dml_kind("WITH cte (SELECT 1) SELECT 1"), None);
    assert_eq!(
        top_level_dml_kind("WITH cte AS NOT (SELECT 1) SELECT 1"),
        None
    );
    assert_eq!(
        top_level_dml_kind(r#"WITH "unclosed AS (SELECT 1) SELECT 1"#),
        None
    );
    assert_eq!(top_level_dml_kind(""), None);
    assert_eq!(top_level_dml_kind("CREATE TABLE items (id int)"), None);
    assert!(recovered_sql_needs_insert_check(Some(
        "WITH cte AS (SELECT 1) MERGE INTO items USING src ON true WHEN MATCHED THEN DELETE"
    )));
}

#[test]
fn select_and_column_write_predicates_fail_closed_on_missing_or_unknown_text() {
    for predicate in [recovered_sql_may_select, recovered_sql_may_write_columns] {
        assert!(predicate(None));
        assert!(predicate(Some("WITH cte AS (SELECT 1")));
        assert!(predicate(Some("VALUES (1)")));
    }
    assert!(recovered_sql_may_select(Some("SELECT id FROM topics")));
    assert!(recovered_sql_may_select(Some(
        "WITH recent AS (SELECT 1) SELECT * FROM recent"
    )));
    for sql in [
        "INSERT INTO t VALUES (1)",
        "UPDATE t SET a = 1",
        "DELETE FROM t",
        "MERGE INTO t USING s ON true WHEN MATCHED THEN DELETE",
    ] {
        assert!(!recovered_sql_may_select(Some(sql)), "{sql}");
    }
    for sql in [
        "INSERT INTO t VALUES (1)",
        "UPDATE t SET a = 1",
        "MERGE INTO t USING s ON true WHEN MATCHED THEN DELETE",
    ] {
        assert!(recovered_sql_may_write_columns(Some(sql)), "{sql}");
    }
    for sql in ["SELECT 1", "DELETE FROM t"] {
        assert!(!recovered_sql_may_write_columns(Some(sql)), "{sql}");
    }
}

#[test]
fn nested_queries_and_writes_are_relevant_but_quoted_keywords_are_not() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/dml-relevance.sql");
    let source = std::fs::read_to_string(root).unwrap();
    let sql: Vec<_> = source.lines().collect();
    assert!(recovered_sql_may_select(Some(sql[0])));
    assert!(recovered_sql_may_write_columns(Some(sql[1])));
    assert!(recovered_sql_may_write_columns(Some(sql[2])));
    assert!(!recovered_sql_may_select(Some(sql[3])));
    assert!(!recovered_sql_may_write_columns(Some(sql[4])));
}
