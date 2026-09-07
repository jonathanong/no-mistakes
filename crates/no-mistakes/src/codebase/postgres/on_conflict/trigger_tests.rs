use super::{judge_file, Catalog};
use crate::codebase::postgres::statements::extract_sql_statement_facts;

fn catalog<'a>(
    triggers: &'a [crate::codebase::postgres::statement_facts::SqlTriggerFact],
    replay_safe: &'a [String],
    trigger_writes: &'a [(String, Vec<String>)],
) -> Catalog<'a> {
    Catalog {
        schema: &[],
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

#[test]
fn constant_assignment_is_not_a_distinctness_noop() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = 'constant'
         WHERE items.note IS DISTINCT FROM EXCLUDED.note;",
    );
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER UPDATE ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    let found = judge_file(&file, &catalog(&triggers, &[], &[]));
    assert!(
        found.iter().any(|(_, message)| message.contains("re-fire")),
        "{found:?}"
    );
}

#[test]
fn delete_only_allowlisted_arbiter_write_is_ignored() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;",
    );
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t BEFORE DELETE ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    let replay_safe = ["audit".to_string()];
    let writes = [("audit".to_string(), vec!["id".to_string()])];
    assert!(judge_file(&file, &catalog(&triggers, &replay_safe, &writes)).is_empty());
}

#[test]
fn named_constraint_assignment_must_be_excluded() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT ON CONSTRAINT items_pkey DO UPDATE SET note = 'x';",
    );
    let found = judge_file(&file, &catalog(&[], &[], &[]));
    assert!(
        found
            .iter()
            .any(|(_, message)| message.contains("named-constraint") || message.contains("arbiter")),
        "{found:?}"
    );
}

#[test]
fn unnamed_and_unknown_arbiters_are_judged() {
    let unnamed =
        extract_sql_statement_facts("INSERT INTO items (id) VALUES (1) ON CONFLICT DO NOTHING;");
    assert!(matches!(
        unnamed.inserts[0].on_conflict.as_ref().unwrap().arbiter,
        crate::codebase::postgres::statement_facts::SqlConflictArbiter::Unknown
    ));
    let mut file = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note;",
    );
    file.inserts[0].on_conflict.as_mut().unwrap().arbiter =
        crate::codebase::postgres::statement_facts::SqlConflictArbiter::Unknown;
    let found = judge_file(&file, &catalog(&[], &[], &[]));
    assert!(
        found
            .iter()
            .any(|(_, message)| message.contains("expression-index")),
        "{found:?}"
    );
}

#[test]
fn do_nothing_skips_triggers_when_disabled() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;",
    );
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    let mut options = catalog(&triggers, &[], &[]);
    options.check_triggers = false;
    assert!(judge_file(&file, &options).is_empty());
}

#[test]
fn update_of_note_does_not_fire_on_other_assignment() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, other) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET other = EXCLUDED.other;",
    );
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER UPDATE OF note ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    assert!(judge_file(&file, &catalog(&triggers, &[], &[])).is_empty());
}

#[test]
fn generated_arbiter_falls_back_to_function_args() {
    use crate::codebase::postgres::types::{
        SqlColumnMetadata, SqlCreateTableMetadata, SqlSchemaFileFacts,
    };
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id) VALUES (1) ON CONFLICT (created_at) DO NOTHING;",
    );
    let schema = [SqlSchemaFileFacts {
        tables: vec![SqlCreateTableMetadata {
            table_name: "items".to_string(),
            columns: vec![SqlColumnMetadata {
                name: "created_at".to_string(),
                type_name: None,
                constraints: Vec::new(),
                is_primary_key: false,
                is_generated: true,
                generated_expression: None,
                generated_function: Some("now".to_string()),
                generated_function_arg_columns: vec!["id".to_string()],
                generated_source_columns: Vec::new(),
            }],
        }],
        ..Default::default()
    }];
    let sources = super::generated::arbiter_source_columns(
        &file.inserts[0],
        file.inserts[0].on_conflict.as_ref().unwrap(),
        &schema,
    );
    assert_eq!(sources, ["id"]);
}
