use super::{extract_sql_statement_facts, SqlOnConflictAction};
use std::path::PathBuf;

#[test]
fn insert_do_nothing() {
    let facts = extract_sql_statement_facts(
        "INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;",
    );
    assert_eq!(facts.inserts.len(), 1);
    let insert = &facts.inserts[0];
    assert_eq!(insert.table, "items");
    assert!(insert.executed);
    assert!(!facts.parse_failed);
    let conflict = insert.on_conflict.as_ref().unwrap();
    assert_eq!(conflict.action, SqlOnConflictAction::DoNothing);
}

#[test]
fn insert_do_update_and_not_exists() {
    let facts = extract_sql_statement_facts(
        "INSERT INTO items (id, note)
         SELECT 1, 'a' WHERE NOT EXISTS (SELECT 1 FROM items WHERE id = 1)
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note;",
    );
    let insert = &facts.inserts[0];
    assert!(insert.guarded_select);
    let conflict = insert.on_conflict.as_ref().unwrap();
    assert_eq!(conflict.action, SqlOnConflictAction::DoUpdate);
    assert_eq!(conflict.assignments[0].column, "note");
}

#[test]
fn explain_without_analyze_is_not_executed() {
    let facts = extract_sql_statement_facts("EXPLAIN INSERT INTO items (id) VALUES (1);");
    assert!(facts.inserts.is_empty());
}

#[test]
fn explain_analyze_is_executed() {
    let facts = extract_sql_statement_facts("EXPLAIN ANALYZE INSERT INTO items (id) VALUES (1);");
    assert_eq!(facts.inserts.len(), 1);
    assert!(facts.inserts[0].executed);
}

#[test]
fn nested_not_exists_is_not_a_guard() {
    let facts = extract_sql_statement_facts(
        "INSERT INTO items (id) VALUES ((SELECT 1 WHERE NOT EXISTS (SELECT 1)));",
    );
    assert_eq!(facts.inserts.len(), 1);
    assert!(!facts.inserts[0].guarded_select);
}

#[test]
fn fallback_counts_insert_keywords() {
    let facts = extract_sql_statement_facts("not sql INSERT INTO a; INSERT INTO b;");
    assert!(facts.parse_failed);
    assert!(facts.insert_keyword_count >= 1);
}

#[test]
fn quoted_insert_is_not_counted() {
    let facts = extract_sql_statement_facts("SELECT 'INSERT INTO x'");
    assert_eq!(facts.insert_keyword_count, 0);
}

#[test]
fn insert_line_skips_comment_and_string_keywords() {
    let sql =
        "-- INSERT INTO decoy\nSELECT 'INSERT INTO decoy';\nINSERT INTO items (id) VALUES (1);";
    let facts = extract_sql_statement_facts(sql);
    assert_eq!(facts.inserts.len(), 1);
    assert_eq!(facts.inserts[0].line, 3);
}

#[test]
fn insert_line_skips_block_comment_and_dollar_quote() {
    let sql = "/* INSERT INTO decoy */\nSELECT $q$INSERT INTO decoy$q$;\nINSERT INTO items (id) VALUES (1);";
    let facts = extract_sql_statement_facts(sql);
    assert_eq!(facts.inserts.len(), 1);
    assert_eq!(facts.inserts[0].line, 3);
}

#[test]
fn trigger_line_skips_comment_keywords() {
    let sql = "-- CREATE TRIGGER decoy BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION f();\nCREATE TRIGGER real AFTER INSERT ON items FOR EACH ROW EXECUTE FUNCTION f();";
    let facts = extract_sql_statement_facts(sql);
    assert_eq!(facts.triggers.len(), 1);
    assert_eq!(facts.triggers[0].line, 2);
}

#[test]
fn omitted_for_each_is_statement_level() {
    let facts =
        extract_sql_statement_facts("CREATE TRIGGER t AFTER INSERT ON items EXECUTE FUNCTION f();");
    assert_eq!(facts.triggers.len(), 1);
    assert!(!facts.triggers[0].for_each_row);
}

#[test]
fn comment_not_exists_is_not_a_top_level_guard() {
    let facts = extract_sql_statement_facts(
        "-- WHERE NOT EXISTS (SELECT 1)\nINSERT INTO items (id) VALUES (1);",
    );
    assert!(!facts.has_top_level_not_exists);
}

#[test]
fn identifier_prefix_is_not_a_not_exists_guard() {
    assert!(!super::has_top_level_not_exists_in(
        "SELECT 1 FROM foowhere not exists (SELECT 1)"
    ));
}

#[test]
fn exists_tautology_is_not_restricted() {
    let facts = extract_sql_statement_facts(
        "SELECT 1 WHERE EXISTS (SELECT 1 FROM t WHERE 1 = 1 UNION ALL SELECT 1 FROM t WHERE id = 1);",
    );
    assert!(facts.selects.iter().any(|select| select
        .exists_set_operations
        .iter()
        .any(|exists| !exists.restricted)));
}

#[test]
fn derived_table_selects_are_collected() {
    let facts =
        extract_sql_statement_facts("SELECT * FROM (SELECT id FROM items WHERE id = 1) AS t");
    assert!(facts
        .selects
        .iter()
        .any(|select| select.tables.iter().any(|table| table == "items")));
}

#[test]
fn data_modifying_cte_insert_is_collected() {
    let facts = extract_sql_statement_facts(
        "WITH added AS (INSERT INTO items (id) VALUES (1) RETURNING id) SELECT * FROM added",
    );
    assert_eq!(facts.inserts.len(), 1);
    assert_eq!(facts.inserts[0].table, "items");
}

#[test]
fn utf8_identifier_does_not_panic_not_exists_scan() {
    assert!(super::has_top_level_not_exists_in(
        "SELECT 1 FROM café WHERE NOT EXISTS (SELECT 1)"
    ));
}

#[test]
fn exists_union_is_unrestricted() {
    let sql = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/statements/exists-union.sql"),
    )
    .unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(facts.selects.iter().any(|select| select
        .exists_set_operations
        .iter()
        .any(|exists| !exists.restricted)));
}

#[test]
fn prepare_inner_insert_is_executed() {
    let facts = extract_sql_statement_facts(
        "PREPARE p AS INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;",
    );
    assert_eq!(facts.inserts.len(), 1);
    assert!(facts.inserts[0].executed);
}

#[test]
fn insert_source_cte_insert_is_collected() {
    let facts = extract_sql_statement_facts(
        "INSERT INTO parent (id)
         WITH added AS (INSERT INTO items (id) VALUES (1) RETURNING id)
         SELECT id FROM added",
    );
    assert_eq!(facts.inserts.len(), 2, "{:#?}", facts.inserts);
    assert!(facts.inserts.iter().any(|insert| insert.table == "parent"));
    assert!(facts.inserts.iter().any(|insert| insert.table == "items"));
}

#[test]
fn nested_join_derived_table_is_collected() {
    let facts = extract_sql_statement_facts(
        "SELECT * FROM (items JOIN (SELECT id FROM accounts WHERE id = 1) AS a ON items.id = a.id)",
    );
    assert!(
        facts
            .selects
            .iter()
            .any(|select| select.tables.iter().any(|table| table == "accounts")),
        "{:#?}",
        facts.selects
    );
}

#[test]
fn existsfoo_identifier_is_not_a_not_exists_guard() {
    assert!(!super::has_top_level_not_exists_in(
        "INSERT INTO items (id) SELECT 1 WHERE NOT EXISTSfoo (SELECT 1)"
    ));
    assert!(super::has_top_level_not_exists_in(
        "INSERT INTO items (id) SELECT 1 WHERE NOT EXISTS (SELECT 1)"
    ));
    assert!(!super::has_top_level_not_exists_in(
        "INSERT INTO items (id) SELECT 1 WHERE NOT EXISTSé (SELECT 1)"
    ));
}

#[test]
fn comment_apostrophe_does_not_hide_insert_keywords() {
    let facts =
        extract_sql_statement_facts("-- '\nINSERT INTO items (id) VALUES (1);\nINSERT INTO");
    assert_eq!(facts.insert_keyword_count, 2, "{facts:?}");
}

#[test]
fn exists_boolean_predicate_is_restricted() {
    let facts = extract_sql_statement_facts(
        "SELECT 1 WHERE EXISTS (
            SELECT 1 FROM accounts WHERE active = TRUE
            UNION ALL
            SELECT 1 FROM accounts WHERE active = FALSE
         );",
    );
    assert!(
        facts.selects.iter().any(|select| select
            .exists_set_operations
            .iter()
            .any(|exists| exists.restricted)),
        "{:#?}",
        facts.selects
    );
}
