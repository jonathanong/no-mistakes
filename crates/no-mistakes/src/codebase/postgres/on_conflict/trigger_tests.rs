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
