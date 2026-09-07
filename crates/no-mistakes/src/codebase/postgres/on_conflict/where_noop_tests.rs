use super::{judge_file, Catalog};
use crate::codebase::postgres::statements::extract_sql_statement_facts;

fn catalog<'a>(
    triggers: &'a [crate::codebase::postgres::statement_facts::SqlTriggerFact],
    trigger_writes: &'a [(String, Vec<String>)],
    replay_safe: &'a [String],
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
fn after_update_where_any_assigned_excluded_column_suppresses_trigger() {
    // Postgres skips the whole DO UPDATE when WHERE is false, so proving one
    // assigned EXCLUDED column is enough even if other SET columns are unproven.
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, a, b) VALUES (1, 'x', 'y')
         ON CONFLICT (id) DO UPDATE SET a = EXCLUDED.a, b = EXCLUDED.b
         WHERE items.b IS DISTINCT FROM EXCLUDED.b;",
    );
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER UPDATE OF a ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    assert!(judge_file(&file, &catalog(&triggers, &[], &[])).is_empty());
}

#[test]
fn trigger_write_of_unproven_column_still_noops() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, a, b) VALUES (1, 'x', 'y')
         ON CONFLICT (id) DO UPDATE SET a = EXCLUDED.a, b = EXCLUDED.b
         WHERE items.b IS DISTINCT FROM EXCLUDED.b;",
    );
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER UPDATE OF a ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    let writes = [("audit".to_string(), vec!["a".to_string()])];
    assert!(judge_file(&file, &catalog(&triggers, &writes, &[])).is_empty());
}

#[test]
fn trigger_rewrite_of_proven_column_is_not_a_noop() {
    // If the trigger rewrites the sole proven column, replay makes WHERE true again.
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, a, b) VALUES (1, 'x', 'y')
         ON CONFLICT (id) DO UPDATE SET a = EXCLUDED.a, b = EXCLUDED.b
         WHERE items.b IS DISTINCT FROM EXCLUDED.b;",
    );
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER UPDATE OF a ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    let writes = [("audit".to_string(), vec!["b".to_string()])];
    let found = judge_file(&file, &catalog(&triggers, &writes, &[]));
    assert!(
        found.iter().any(|(_, message)| message.contains("re-fire")),
        "{found:?}"
    );
}

#[test]
fn where_on_unassigned_column_does_not_prove_a_noop() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, a, b) VALUES (1, 'x', 'y')
         ON CONFLICT (id) DO UPDATE SET a = EXCLUDED.a, b = EXCLUDED.b
         WHERE items.note IS DISTINCT FROM EXCLUDED.note;",
    );
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER t AFTER UPDATE OF a ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    let found = judge_file(&file, &catalog(&triggers, &[], &[]));
    assert!(
        found.iter().any(|(_, message)| message.contains("re-fire")),
        "{found:?}"
    );
}

#[test]
fn empty_assigned_list_does_not_prove_a_noop() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, b) VALUES (1, 'y')
         ON CONFLICT (id) DO UPDATE SET b = EXCLUDED.b
         WHERE items.b IS DISTINCT FROM EXCLUDED.b;",
    );
    let conflict = file.inserts[0].on_conflict.as_ref().unwrap();
    assert!(!super::where_noop::where_proves_noop(
        conflict,
        &[],
        "items",
        &catalog(&[], &[], &[]),
        &[],
    ));
}

#[test]
fn other_applicable_trigger_rewrite_is_not_a_noop() {
    // An allowlisted BEFORE trigger can rewrite `b` so the AFTER trigger still
    // fires on replay even though that AFTER function does not write `b`.
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id, a, b) VALUES (1, 'x', 'y')
         ON CONFLICT (id) DO UPDATE SET a = EXCLUDED.a, b = EXCLUDED.b
         WHERE items.b IS DISTINCT FROM EXCLUDED.b;",
    );
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER n BEFORE UPDATE OF a ON items FOR EACH ROW EXECUTE FUNCTION normalize();
         CREATE TRIGGER t AFTER UPDATE OF a ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    let writes = [("normalize".to_string(), vec!["b".to_string()])];
    let replay_safe = ["normalize".to_string()];
    let found = judge_file(&file, &catalog(&triggers, &writes, &replay_safe));
    assert!(
        found.iter().any(|(_, message)| message.contains("audit")),
        "{found:?}"
    );
}
