use super::dml_kind::{recovered_sql_needs_insert_check, top_level_dml_kind, TopLevelDml};

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
