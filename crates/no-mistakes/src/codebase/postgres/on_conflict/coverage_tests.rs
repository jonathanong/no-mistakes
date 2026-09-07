use super::{form_is_self, judge_file, Catalog};
use crate::codebase::postgres::statement_facts::{SqlStatementFileFacts, SqlValueForm};
use crate::codebase::postgres::statements::extract_sql_statement_facts;
use crate::codebase::postgres::types::{
    SqlColumnMetadata, SqlCreateTableMetadata, SqlSchemaFileFacts,
};

fn catalog<'a>(schema: &'a [SqlSchemaFileFacts]) -> Catalog<'a> {
    Catalog {
        schema,
        triggers: &[],
        replay_safe: &[],
        check_convergence: true,
        check_volatility: true,
        check_arbiter: true,
        check_triggers: true,
        check_generated: true,
        trigger_writes: &[],
    }
}

#[test]
fn unparseable_single_insert_without_guard_fails_closed() {
    let found = judge_file(
        &extract_sql_statement_facts("not sql INSERT INTO items;"),
        &catalog(&[]),
    );
    assert!(
        found
            .iter()
            .any(|(_, message)| message.contains("could not be proven replay-safe")),
        "{found:?}"
    );
}

#[test]
fn unexecuted_insert_is_skipped() {
    let mut file = extract_sql_statement_facts("INSERT INTO items (id) VALUES (1);");
    file.inserts[0].executed = false;
    assert!(judge_file(&file, &catalog(&[])).is_empty());
}

#[test]
fn parse_failed_zero_origin_line_uses_line_one() {
    let file = SqlStatementFileFacts {
        parse_failed: true,
        insert_keyword_count: 1,
        origin_line: 0,
        ..Default::default()
    };
    let found = judge_file(&file, &catalog(&[]));
    assert_eq!(found[0].0, 1);
}

#[test]
fn form_is_self_matches_column_names() {
    assert!(form_is_self(
        &SqlValueForm::SelfRef {
            column: "Note".to_string()
        },
        "note"
    ));
    assert!(!form_is_self(
        &SqlValueForm::SelfRef {
            column: "id".to_string()
        },
        "note"
    ));
}

#[test]
fn generated_sources_empty_for_non_column_arbiter() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id) VALUES (1) ON CONFLICT ON CONSTRAINT items_pkey DO NOTHING;",
    );
    let sources = super::generated::arbiter_source_columns(
        &file.inserts[0],
        file.inserts[0].on_conflict.as_ref().unwrap(),
        &[],
    );
    assert!(sources.is_empty());
}

#[test]
fn empty_schema_skips_generated_arbiter_check() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note;",
    );
    let schema = [SqlSchemaFileFacts {
        tables: vec![],
        ..Default::default()
    }];
    assert!(judge_file(&file, &catalog(&schema)).is_empty());
}

#[test]
fn generated_self_assignment_of_source_is_safe() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, updated_at) VALUES (1, '2020-01-01')
         ON CONFLICT (created_at) DO UPDATE SET updated_at = updated_at;",
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
                generated_function: None,
                generated_function_arg_columns: Vec::new(),
                generated_source_columns: vec!["updated_at".to_string()],
            }],
        }],
        ..Default::default()
    }];
    assert!(judge_file(&file, &catalog(&schema)).is_empty());
}

#[test]
fn guarded_select_still_rejects_statement_insert_triggers() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id) SELECT 1 WHERE NOT EXISTS (SELECT 1 FROM items WHERE id = 1);",
    );
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER INSERT ON items EXECUTE FUNCTION audit();",
    )
    .triggers;
    let mut options = catalog(&[]);
    options.triggers = &triggers;
    let found = judge_file(&file, &options);
    assert!(
        found
            .iter()
            .any(|(_, message)| message.contains("statement-level")),
        "{found:?}"
    );
    let replay_safe = ["audit".to_string()];
    options.replay_safe = &replay_safe;
    assert!(judge_file(&file, &options).is_empty());
}
