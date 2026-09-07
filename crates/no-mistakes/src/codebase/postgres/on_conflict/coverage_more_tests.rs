use super::{judge_file, Catalog};
use crate::codebase::postgres::statement_facts::{
    SqlAssignmentFact, SqlConflictArbiter, SqlInsertFact, SqlOnConflictAction, SqlOnConflictFact,
    SqlValueForm,
};
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

fn conflict(form: SqlValueForm) -> SqlOnConflictFact {
    SqlOnConflictFact {
        action: SqlOnConflictAction::DoUpdate,
        arbiter: SqlConflictArbiter::Columns(vec!["id".into()]),
        assignments: vec![SqlAssignmentFact {
            column: "note".into(),
            form,
        }],
        where_proof: Default::default(),
    }
}

#[test]
fn convergence_judge_covers_disabled_flag_arms() {
    let subquery = conflict(SqlValueForm::Subquery);
    assert!(super::convergence::judge(&subquery, false, true).is_none());
    assert!(super::convergence::judge(&subquery, true, false).is_some());
    let placeholder = conflict(SqlValueForm::Placeholder);
    assert!(super::convergence::judge(&placeholder, false, true).is_none());
    let volatile = conflict(SqlValueForm::Volatile { name: "now".into() });
    assert!(super::convergence::judge(&volatile, true, false).is_none());
    let self_ref = conflict(SqlValueForm::SelfRef {
        column: "id".into(),
    });
    assert!(super::convergence::judge(&self_ref, false, true).is_none());
    let other = conflict(SqlValueForm::Other);
    assert!(super::convergence::judge(&other, false, true).is_none());
    let greatest = conflict(SqlValueForm::Greatest {
        args: vec![SqlValueForm::Placeholder],
    });
    assert!(super::convergence::judge(&greatest, false, false).is_none());
    let coalesce = conflict(SqlValueForm::Coalesce {
        args: vec![SqlValueForm::SelfRef {
            column: "id".into(),
        }],
    });
    let found = super::convergence::judge(&coalesce, true, false).unwrap();
    assert!(found.contains("different column"), "{found}");
}

#[test]
fn arbiter_self_assignments_are_safe() {
    let insert = SqlInsertFact {
        table: "items".into(),
        line: 1,
        executed: true,
        on_conflict: None,
        guarded_select: false,
        assignments: Vec::new(),
    };
    let self_form = SqlValueForm::SelfRef {
        column: "note".into(),
    };
    for arbiter in [
        SqlConflictArbiter::Columns(vec!["note".into()]),
        SqlConflictArbiter::Constraint("items_pkey".into()),
        SqlConflictArbiter::Unknown,
    ] {
        let conflict = SqlOnConflictFact {
            action: SqlOnConflictAction::DoUpdate,
            arbiter,
            assignments: vec![SqlAssignmentFact {
                column: "note".into(),
                form: self_form.clone(),
            }],
            where_proof: Default::default(),
        };
        assert!(super::arbiter::judge(&insert, &conflict).is_none());
    }
}

#[test]
fn generated_judge_skips_non_column_arbiters_and_unrelated_generated_cols() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT ON CONSTRAINT items_pkey DO UPDATE SET note = EXCLUDED.note;",
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
                generated_function_arg_columns: vec!["id".to_string()],
                generated_source_columns: Vec::new(),
            }],
        }],
        ..Default::default()
    }];
    assert!(judge_file(&file, &catalog(&schema)).is_empty());
    let columns = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note;",
    );
    assert!(judge_file(&columns, &catalog(&schema)).is_empty());
    let generated = extract_sql_statement_facts(
        "INSERT INTO items (id) VALUES (1)
         ON CONFLICT (created_at) DO UPDATE SET id = EXCLUDED.id;",
    );
    let found = judge_file(&generated, &catalog(&schema));
    assert!(
        found
            .iter()
            .any(|(_, message)| message.contains("generated arbiter")),
        "{found:?}"
    );
}

#[test]
fn allowlisted_row_triggers_and_unknown_arbiters_are_judged() {
    let before = extract_sql_statement_facts(
        "INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;",
    );
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    let replay_safe = ["audit".to_string()];
    let mut options = catalog(&[]);
    options.triggers = &triggers;
    options.replay_safe = &replay_safe;
    assert!(judge_file(&before, &options).is_empty());
    let after_insert = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note;",
    );
    let insert_only = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    options.replay_safe = &[];
    options.triggers = &insert_only;
    assert!(judge_file(&after_insert, &options).is_empty());
    let update = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER UPDATE ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    options.triggers = &update;
    options.replay_safe = &replay_safe;
    assert!(judge_file(&after_insert, &options).is_empty());
    let mut unknown = after_insert.clone();
    unknown.inserts[0].on_conflict.as_mut().unwrap().arbiter = SqlConflictArbiter::Unknown;
    options.check_arbiter = false;
    options.replay_safe = &[];
    let found = judge_file(&unknown, &options);
    assert!(
        found.iter().any(|(_, message)| message.contains("re-fire")),
        "{found:?}"
    );
}

#[test]
fn disjunctive_where_does_not_prove_a_noop() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, note) VALUES (1, 'a')
         ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note
         WHERE items.note IS DISTINCT FROM EXCLUDED.note OR items.id IS NULL;",
    );
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER UPDATE ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    let mut options = catalog(&[]);
    options.triggers = &triggers;
    let found = judge_file(&file, &options);
    assert!(
        found.iter().any(|(_, message)| message.contains("re-fire")),
        "{found:?}"
    );
}
