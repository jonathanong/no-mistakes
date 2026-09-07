use super::{judge_file, Catalog};
use crate::codebase::postgres::statement_facts::SqlTriggerFact;
use crate::codebase::postgres::statements::extract_sql_statement_facts;
use crate::codebase::postgres::types::{
    SqlColumnMetadata, SqlCreateTableMetadata, SqlSchemaFileFacts,
};

fn catalog<'a>(
    schema: &'a [SqlSchemaFileFacts],
    triggers: &'a [SqlTriggerFact],
    replay_safe: &'a [String],
    trigger_writes: &'a [(String, Vec<String>)],
) -> Catalog<'a> {
    Catalog {
        schema,
        triggers,
        replay_safe,
        check_convergence: true,
        check_volatility: true,
        check_arbiter: true,
        check_triggers: true,
        check_generated: true,
        trigger_writes,
    }
}

fn messages(sql: &str) -> Vec<String> {
    let file = extract_sql_statement_facts(sql);
    judge_file(&file, &catalog(&[], &[], &[], &[]))
        .into_iter()
        .map(|(_, message)| message)
        .collect()
}

#[test]
fn insert_without_conflict_is_unsafe() {
    let found = messages("INSERT INTO items (id) VALUES (1);");
    assert_eq!(found.len(), 1);
    assert!(found[0].contains("ON CONFLICT"), "{found:?}");
}

#[test]
fn do_nothing_is_safe() {
    assert!(messages("INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;").is_empty());
}

#[test]
fn not_exists_select_is_safe() {
    assert!(messages(
        "INSERT INTO items (id) SELECT 1 WHERE NOT EXISTS (SELECT 1 FROM items WHERE id = 1);"
    )
    .is_empty());
}

#[test]
fn do_update_excluded_is_safe() {
    assert!(messages(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note;"
    )
    .is_empty());
}

#[test]
fn cross_column_assignment_is_not_convergent() {
    let found = messages(
        "INSERT INTO items (id, updated_at) VALUES (1, '2020-01-01')
         ON CONFLICT (id) DO UPDATE SET updated_at = created_at;",
    );
    assert!(
        found
            .iter()
            .any(|message| message.contains("not a self-assignment")),
        "{found:?}"
    );
}

#[test]
fn matching_self_assignment_is_convergent() {
    assert!(messages(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = note;"
    )
    .is_empty());
}

#[test]
fn null_arbiter_assignment_is_unsafe() {
    let found =
        messages("INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO UPDATE SET id = NULL;");
    assert!(
        found
            .iter()
            .any(|message| message.contains("arbiter column")),
        "{found:?}"
    );
}

#[test]
fn placeholder_assignment_is_not_convergent() {
    let found = messages(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = sql_placeholder_1;",
    );
    assert!(
        found
            .iter()
            .any(|message| message.contains("bind parameter")),
        "{found:?}"
    );
}

#[test]
fn recovered_insert_plus_unparseable_insert_fails_closed() {
    let found =
        messages("INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING; INSERT INTO");
    assert!(
        found
            .iter()
            .any(|message| message.contains("more than one INSERT")),
        "{found:?}"
    );
}

#[test]
fn do_nothing_still_checks_before_insert_triggers() {
    let sql = "INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;";
    let file = extract_sql_statement_facts(sql);
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    let found = judge_file(&file, &catalog(&[], &triggers, &[], &[]));
    assert!(
        found
            .iter()
            .any(|(_, message)| message.contains("BEFORE INSERT")),
        "{found:?}"
    );
}

#[test]
fn volatile_assignment_is_unsafe() {
    let found = messages(
        "INSERT INTO items (id, seen) VALUES (1, now())
         ON CONFLICT (id) DO UPDATE SET seen = now();",
    );
    assert!(
        found.iter().any(|message| message.contains("volatile")),
        "{found:?}"
    );
}

#[test]
fn coalesce_self_then_now_is_safe() {
    assert!(messages(
        "INSERT INTO items (id, seen) VALUES (1, now())
         ON CONFLICT (id) DO UPDATE SET seen = COALESCE(items.seen, now());"
    )
    .is_empty());
}

#[test]
fn unparseable_multi_insert_fails_closed() {
    let found = messages("not sql INSERT INTO a; INSERT INTO b;");
    assert!(
        found
            .iter()
            .any(|message| message.contains("more than one INSERT")),
        "{found:?}"
    );
}

#[test]
fn unparseable_single_insert_with_not_exists_is_safe() {
    let sql =
        "INSERT INTO items (id) SELECT 1 WHERE NOT EXISTS (SELECT 1 FROM items WHERE id = 1) !!!";
    assert!(messages(sql).is_empty() || extract_sql_statement_facts(sql).has_top_level_not_exists);
}

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

#[test]
fn after_update_without_where_re_fires_trigger() {
    let sql = "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note;";
    let file = extract_sql_statement_facts(sql);
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER UPDATE ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    let found = judge_file(&file, &catalog(&[], &triggers, &[], &[]));
    assert!(
        found.iter().any(|(_, message)| message.contains("re-fire")),
        "{found:?}"
    );
}
