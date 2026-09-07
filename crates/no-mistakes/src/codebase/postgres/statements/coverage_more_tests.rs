use super::extract_sql_statement_facts;

#[test]
fn parenthesized_and_unary_not_exists_guard_selects() {
    for sql in [
        "INSERT INTO items (id) SELECT 1 WHERE (NOT EXISTS (SELECT 1 FROM items WHERE id = 1))",
        "INSERT INTO items (id) SELECT 1 WHERE NOT (EXISTS (SELECT 1 FROM items WHERE id = 1))",
        "INSERT INTO items (id) SELECT 1 WHERE id = 1 AND NOT EXISTS (SELECT 1 FROM items)",
        "INSERT INTO items (id) (SELECT 1 WHERE NOT EXISTS (SELECT 1 FROM items))",
    ] {
        let facts = extract_sql_statement_facts(sql);
        assert!(
            facts.inserts.iter().any(|insert| insert.guarded_select)
                || facts.has_top_level_not_exists,
            "{sql} {:#?}",
            facts.inserts
        );
    }
    let unguarded = extract_sql_statement_facts("INSERT INTO items (id) SELECT 1 WHERE id = 1");
    assert!(unguarded
        .inserts
        .iter()
        .all(|insert| !insert.guarded_select));
    let mixed = extract_sql_statement_facts(
        "INSERT INTO items (id) SELECT 1 WHERE NOT EXISTS (SELECT 1)
         UNION ALL
         SELECT 2",
    );
    assert!(
        mixed.inserts.is_empty() || mixed.inserts.iter().any(|insert| !insert.guarded_select),
        "{:#?}",
        mixed.inserts
    );
}

#[test]
fn exists_wrappers_values_and_placeholder_bounds() {
    let nested = extract_sql_statement_facts(
        "SELECT 1 WHERE EXISTS (
            (SELECT 1 FROM accounts WHERE id = 1
             UNION ALL
             SELECT 1 FROM accounts WHERE id = 2)
         )",
    );
    assert!(
        nested
            .selects
            .iter()
            .any(|select| !select.exists_set_operations.is_empty()),
        "{:#?}",
        nested.selects
    );
    let unary = extract_sql_statement_facts(
        "SELECT 1 WHERE NOT EXISTS (
            SELECT 1 FROM accounts WHERE id = sql_placeholder_1
            UNION ALL
            SELECT 1 FROM accounts WHERE id = 'x'
         )",
    );
    assert!(!unary.selects.is_empty());
    let subquery = extract_sql_statement_facts(
        "SELECT 1 FROM items WHERE (SELECT 1 FROM accounts WHERE EXISTS (
            SELECT 1 FROM t WHERE t.id = 1 UNION SELECT 1 FROM t WHERE t.id = 2
         ))",
    );
    assert!(
        subquery
            .selects
            .iter()
            .any(|select| !select.exists_set_operations.is_empty()),
        "{:#?}",
        subquery.selects
    );
    let values = extract_sql_statement_facts("SELECT 1 WHERE EXISTS (VALUES (1))");
    assert!(
        values
            .selects
            .iter()
            .all(|select| select.exists_set_operations.is_empty())
            || values.selects.is_empty()
    );
    let mixed_values = extract_sql_statement_facts(
        "SELECT 1 WHERE EXISTS (
            SELECT 1 FROM accounts WHERE id = 1
            UNION ALL
            VALUES (1)
         )",
    );
    assert!(
        mixed_values.selects.iter().any(|select| select
            .exists_set_operations
            .iter()
            .any(|exists| !exists.restricted)),
        "{:#?}",
        mixed_values.selects
    );
}

#[test]
fn join_forms_without_on_do_not_record_join_predicates() {
    for sql in [
        "SELECT * FROM items CROSS JOIN accounts",
        "SELECT * FROM items NATURAL JOIN accounts",
        "SELECT * FROM items JOIN accounts USING (id)",
    ] {
        let facts = extract_sql_statement_facts(sql);
        assert!(
            facts.selects.iter().any(|select| {
                select.tables.contains(&"items".to_string()) && select.predicate_sql.is_empty()
            }),
            "{sql} {:#?}",
            facts.selects
        );
    }
}

#[test]
fn create_table_is_not_a_select_and_values_are_not_set_inserts() {
    let table = extract_sql_statement_facts("CREATE TABLE items (id int);");
    assert!(table.selects.is_empty());
    let values = extract_sql_statement_facts("INSERT INTO items (id) VALUES (1);");
    assert_eq!(values.inserts.len(), 1);
}

#[test]
fn conflict_where_swapped_and_mismatched_null_proofs() {
    let swapped = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note
         WHERE EXCLUDED.note IS NOT NULL AND items.note IS NULL;",
    );
    let proof = &swapped.inserts[0].on_conflict.as_ref().unwrap().where_proof;
    assert!(
        proof
            .null_and_excluded_not_null
            .iter()
            .any(|column| column.eq_ignore_ascii_case("note")),
        "{proof:?}"
    );
    let mismatched = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note
         WHERE items.note IS NULL AND EXCLUDED.id IS NOT NULL;",
    );
    let proof = &mismatched.inserts[0]
        .on_conflict
        .as_ref()
        .unwrap()
        .where_proof;
    assert!(proof.null_and_excluded_not_null.is_empty(), "{proof:?}");
    let not_leaf = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note
         WHERE items.note IS NULL AND NOT (EXCLUDED.note = 1);",
    );
    assert!(not_leaf.inserts[0]
        .on_conflict
        .as_ref()
        .unwrap()
        .where_proof
        .null_and_excluded_not_null
        .is_empty());
}

#[test]
fn conflict_where_or_does_not_classify_nested_leaves() {
    let facts = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note
         WHERE flag OR (items.note IS NULL AND EXCLUDED.note IS NOT NULL)
            OR items.note IS DISTINCT FROM EXCLUDED.note;",
    );
    let proof = &facts.inserts[0].on_conflict.as_ref().unwrap().where_proof;
    assert!(proof.disjunctive, "{proof:?}");
    assert!(
        proof.distinct_from_excluded.is_empty() || proof.disjunctive,
        "{proof:?}"
    );
}
