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
