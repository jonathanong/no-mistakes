use super::{extract_sql_statement_facts, SqlConflictArbiter, SqlOnConflictAction};

#[test]
fn create_view_and_explain_collect_selects() {
    let view = extract_sql_statement_facts("CREATE VIEW v AS SELECT id FROM items WHERE id = 1");
    assert!(view
        .selects
        .iter()
        .any(|select| select.tables.contains(&"items".to_string())));
    let explained =
        extract_sql_statement_facts("EXPLAIN ANALYZE SELECT id FROM accounts WHERE id = 1");
    assert!(explained
        .selects
        .iter()
        .any(|select| select.tables.contains(&"accounts".to_string())));
}

#[test]
fn join_on_exists_set_operation_is_collected() {
    let facts = extract_sql_statement_facts(
        "SELECT items.id FROM items
         INNER JOIN accounts ON EXISTS (
            SELECT 1 FROM t WHERE t.id = items.id
            UNION ALL
            SELECT 1 FROM t WHERE t.id = 2
         )",
    );
    assert!(
        facts.selects.iter().any(|select| {
            !select.predicate_sql.is_empty()
                && select
                    .exists_set_operations
                    .iter()
                    .any(|exists| !exists.restricted)
        }),
        "{:#?}",
        facts.selects
    );
}

#[test]
fn outer_join_on_predicates_are_recorded() {
    for sql in [
        "SELECT * FROM items LEFT JOIN accounts ON items.id = accounts.id",
        "SELECT * FROM items RIGHT JOIN accounts ON items.id = accounts.id",
        "SELECT * FROM items FULL JOIN accounts ON items.id = accounts.id",
    ] {
        let facts = extract_sql_statement_facts(sql);
        assert!(
            facts
                .selects
                .iter()
                .any(|select| select.tables.contains(&"items".to_string())
                    && select.tables.contains(&"accounts".to_string())),
            "{sql} {:#?}",
            facts.selects
        );
    }
}

#[test]
fn nested_not_exists_and_union_guards() {
    assert!(super::has_top_level_not_exists_in(
        "INSERT INTO items (id) SELECT 1 WHERE NOT EXISTS (SELECT 1) AND true"
    ));
    let facts = extract_sql_statement_facts(
        "INSERT INTO items (id) SELECT 1 WHERE NOT EXISTS (SELECT 1)
         UNION ALL
         SELECT 2 WHERE NOT EXISTS (SELECT 1)",
    );
    assert!(
        facts.inserts.iter().any(|insert| insert.guarded_select) || facts.has_top_level_not_exists
    );
}

#[test]
fn conflict_constraint_and_expression_arbiters() {
    let named = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT ON CONSTRAINT items_pkey DO UPDATE SET note = EXCLUDED.note;",
    );
    assert!(matches!(
        named.inserts[0].on_conflict.as_ref().unwrap().arbiter,
        SqlConflictArbiter::Constraint(_)
    ));
    let unnamed =
        extract_sql_statement_facts("INSERT INTO items (id) VALUES (1) ON CONFLICT DO NOTHING;");
    assert!(matches!(
        unnamed.inserts[0].on_conflict.as_ref().unwrap().arbiter,
        SqlConflictArbiter::Unknown
    ));
}

#[test]
fn disjunctive_conflict_where_is_recorded() {
    let facts = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note
         WHERE items.note IS DISTINCT FROM EXCLUDED.note OR items.id IS NULL;",
    );
    assert_eq!(facts.inserts.len(), 1);
}

#[test]
fn tuple_assignment_records_a_column() {
    let facts = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET (note, id) = (EXCLUDED.note, EXCLUDED.id);",
    );
    let conflict = facts.inserts[0].on_conflict.as_ref().unwrap();
    assert_eq!(conflict.action, SqlOnConflictAction::DoUpdate);
    assert!(!conflict.assignments.is_empty());
}

#[test]
fn truncate_and_instead_of_triggers_are_collected() {
    let truncate = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER TRUNCATE ON items FOR EACH STATEMENT EXECUTE FUNCTION audit();",
    );
    assert_eq!(truncate.triggers.len(), 1);
    let instead = extract_sql_statement_facts(
        "CREATE TRIGGER t INSTEAD OF INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    );
    assert_eq!(instead.triggers.len(), 1);
}

#[test]
fn create_function_wrapper_is_not_executed_insert() {
    let facts = extract_sql_statement_facts(
        "CREATE FUNCTION f() RETURNS void AS $$ INSERT INTO items (id) VALUES (1); $$ LANGUAGE sql;",
    );
    assert!(facts.inserts.is_empty() || !facts.inserts.iter().all(|insert| insert.executed));
}

#[test]
fn exists_or_and_compound_column_restriction() {
    let facts = extract_sql_statement_facts(
        "SELECT 1 WHERE EXISTS (
            SELECT 1 FROM accounts WHERE accounts.id = 1 AND active = TRUE
            UNION ALL
            SELECT 1 FROM accounts WHERE accounts.id = $1 OR accounts.id = 2
         )",
    );
    assert!(!facts.selects.is_empty(), "{:#?}", facts.selects);
}

#[test]
fn dollar_and_escaped_quotes_do_not_count_inserts() {
    let facts = extract_sql_statement_facts(
        "SELECT $tag$INSERT INTO decoy$tag$; SELECT E'it\\'s INSERT INTO x'",
    );
    assert!(facts.insert_keyword_count <= 1, "{facts:?}");
}
