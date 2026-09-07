use super::{judge_file, Catalog};
use crate::codebase::postgres::statements::extract_sql_statement_facts;

fn catalog<'a>(
    triggers: &'a [crate::codebase::postgres::statement_facts::SqlTriggerFact],
) -> Catalog<'a> {
    Catalog {
        schema: &[],
        triggers,
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
    assert!(judge_file(&file, &catalog(&triggers)).is_empty());
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
    let found = judge_file(&file, &catalog(&triggers));
    assert!(
        found.iter().any(|(_, message)| message.contains("re-fire")),
        "{found:?}"
    );
}
