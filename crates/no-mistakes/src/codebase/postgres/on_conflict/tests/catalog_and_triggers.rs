#[test]
fn missing_schema_table_fails_generated_check() {
    let sql = "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note;";
    let file = extract_sql_statement_facts(sql);
    let schema = [SqlSchemaFileFacts {
        tables: vec![SqlCreateTableMetadata {
            table_name: "other".to_string(),
            columns: vec![],
        }],
        ..Default::default()
    }];
    let found = judge_file(&file, &catalog(&schema, &[], &[], &[]));
    assert!(
        found
            .iter()
            .any(|(_, message)| message.contains("missing from schema")),
        "{found:?}"
    );
}

#[test]
fn generated_arbiter_source_assignment_fails() {
    let sql = "INSERT INTO items (id, updated_at) VALUES (1, '2020-01-01')
         ON CONFLICT (created_at) DO UPDATE SET updated_at = EXCLUDED.updated_at;";
    let file = extract_sql_statement_facts(sql);
    let schema = [SqlSchemaFileFacts {
        tables: vec![SqlCreateTableMetadata {
            table_name: "items".to_string(),
            columns: vec![SqlColumnMetadata {
                name: "created_at".to_string(),
                is_generated: true,
                generated_source_columns: vec!["updated_at".to_string()],
                ..column()
            }],
        }],
        ..Default::default()
    }];
    let found = judge_file(&file, &catalog(&schema, &[], &[], &[]));
    assert!(
        found
            .iter()
            .any(|(_, message)| message.contains("generated arbiter")),
        "{found:?}"
    );
}

fn column() -> SqlColumnMetadata {
    SqlColumnMetadata {
        name: String::new(),
        type_name: None,
        constraints: Vec::new(),
        is_primary_key: false,
        is_generated: false,
        generated_expression: None,
        generated_function: None,
        generated_function_arg_columns: Vec::new(),
        generated_source_columns: Vec::new(),
    }
}

#[test]
fn distinct_from_excluded_where_is_recorded() {
    let facts = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note
         WHERE items.note IS DISTINCT FROM EXCLUDED.note;",
    );
    let proof = &facts.inserts[0].on_conflict.as_ref().unwrap().where_proof;
    assert!(
        proof
            .distinct_from_excluded
            .iter()
            .any(|column| column.eq_ignore_ascii_case("note")),
        "{proof:?}"
    );
}

#[test]
fn null_and_excluded_not_null_where_is_recorded() {
    let facts = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note
         WHERE items.note IS NULL AND EXCLUDED.note IS NOT NULL;",
    );
    let proof = &facts.inserts[0].on_conflict.as_ref().unwrap().where_proof;
    assert!(
        proof
            .null_and_excluded_not_null
            .iter()
            .any(|column| column.eq_ignore_ascii_case("note")),
        "{proof:?}"
    );
}

#[test]
fn nested_conflict_where_still_records_distinct() {
    let facts = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note
         WHERE (items.note IS DISTINCT FROM EXCLUDED.note);",
    );
    let proof = &facts.inserts[0].on_conflict.as_ref().unwrap().where_proof;
    assert!(
        proof
            .distinct_from_excluded
            .iter()
            .any(|column| column.eq_ignore_ascii_case("note")),
        "{proof:?}"
    );
}

#[test]
fn volatility_only_ignores_placeholder_convergence() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = sql_placeholder_1;",
    );
    let mut options = catalog(&[], &[], &[], &[]);
    options.check_convergence = false;
    options.check_volatility = true;
    assert!(judge_file(&file, &options).is_empty());
}

#[test]
fn statement_level_trigger_is_unsafe_on_do_update() {
    let sql = "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note;";
    let file = extract_sql_statement_facts(sql);
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER INSERT ON items EXECUTE FUNCTION audit();",
    )
    .triggers;
    let found = judge_file(&file, &catalog(&[], &triggers, &[], &[]));
    assert!(
        found
            .iter()
            .any(|(_, message)| message.contains("statement-level")),
        "{found:?}"
    );
}

#[test]
fn after_update_where_distinct_suppresses_trigger() {
    let sql = "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note
         WHERE items.note IS DISTINCT FROM EXCLUDED.note;";
    let file = extract_sql_statement_facts(sql);
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER UPDATE ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    assert!(judge_file(&file, &catalog(&[], &triggers, &[], &[])).is_empty());
}

#[test]
fn not_is_null_excluded_counts_as_not_null_proof() {
    let facts = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note
         WHERE items.note IS NULL AND NOT EXCLUDED.note IS NULL;",
    );
    let proof = &facts.inserts[0].on_conflict.as_ref().unwrap().where_proof;
    assert!(
        proof
            .null_and_excluded_not_null
            .iter()
            .any(|column| column.eq_ignore_ascii_case("note")),
        "{proof:?}"
    );
}

#[test]
fn allowlisted_statement_trigger_is_safe() {
    let sql = "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note;";
    let file = extract_sql_statement_facts(sql);
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER INSERT ON items EXECUTE FUNCTION audit();",
    )
    .triggers;
    let replay_safe = ["audit".to_string()];
    assert!(judge_file(&file, &catalog(&[], &triggers, &replay_safe, &[])).is_empty());
}
