use super::{judge_file, Catalog};
use crate::codebase::postgres::statement_facts::{
    SqlTriggerEvent, SqlTriggerFact, SqlTriggerPeriod,
};
use crate::codebase::postgres::statements::extract_sql_statement_facts;

fn catalog<'a>(
    triggers: &'a [SqlTriggerFact],
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

const AFTER_A: &str =
    "CREATE TRIGGER t AFTER UPDATE OF a ON items FOR EACH ROW EXECUTE FUNCTION audit();";
const DO_UPDATE: &str = "ON CONFLICT (id) DO UPDATE SET a = EXCLUDED.a, b = EXCLUDED.b \
     WHERE items.b IS DISTINCT FROM EXCLUDED.b;";

fn sql(prefix: &str) -> String {
    format!("{prefix} {DO_UPDATE}")
}

fn assert_refires(insert: &str) {
    let found = judge_file(
        &extract_sql_statement_facts(&sql(insert)),
        &catalog(&extract_sql_statement_facts(AFTER_A).triggers, &[], &[]),
    );
    assert!(
        found.iter().any(|(_, message)| message.contains("re-fire")),
        "{found:?}"
    );
}

#[test]
fn volatile_insert_value_is_not_a_noop() {
    assert_refires("INSERT INTO items (id, a, b) VALUES (1, 'x', now())");
}

#[test]
fn insert_select_volatile_is_not_a_noop() {
    assert_refires("INSERT INTO items (id, a, b) SELECT 1, 'x', now() FROM (VALUES (1)) AS src");
}

#[test]
fn omitted_insert_column_is_not_a_noop() {
    assert_refires("INSERT INTO items (id, a) VALUES (1, 'x')");
}

#[test]
fn implicit_column_list_is_not_a_noop() {
    assert_refires("INSERT INTO items VALUES (1, 'x', 'y')");
}

#[test]
fn select_star_is_not_a_noop() {
    assert_refires("INSERT INTO items (id, a, b) SELECT * FROM (SELECT 1, 'x', 'y') AS src");
}

#[test]
fn select_column_ref_is_not_a_noop() {
    assert_refires("INSERT INTO items (id, a, b) SELECT id, a, b FROM src");
}

#[test]
fn default_insert_value_is_not_a_noop() {
    assert_refires("INSERT INTO items (id, a, b) VALUES (1, 'x', DEFAULT)");
}

#[test]
fn current_timestamp_ident_is_not_a_noop() {
    assert_refires("INSERT INTO items (id, a, b) VALUES (1, 'x', CURRENT_TIMESTAMP)");
}

#[test]
fn relative_datetime_literal_is_not_a_noop() {
    assert_refires("INSERT INTO items (id, a, b) VALUES (1, 'x', 'now')");
    assert_refires("INSERT INTO items (id, a, b) VALUES (1, 'x', N'now')");
}

#[test]
fn overriding_user_value_is_not_a_noop() {
    let found = judge_file(
        &extract_sql_statement_facts(&sql("INSERT INTO items (id, a, b) OVERRIDING /* skip */
             USER VALUE VALUES (1, 'x', 'y')")),
        &catalog(&extract_sql_statement_facts(AFTER_A).triggers, &[], &[]),
    );
    assert!(
        found
            .iter()
            .any(|(_, message)| message.contains("re-fire") || message.contains("replay-safe")),
        "{found:?}"
    );
}

#[test]
fn after_insert_rewrite_is_not_a_noop() {
    let file =
        extract_sql_statement_facts(&sql("INSERT INTO items (id, a, b) VALUES (1, 'x', 'y')"));
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER n AFTER INSERT ON items FOR EACH STATEMENT EXECUTE FUNCTION normalize();
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

#[test]
fn coalesce_volatile_insert_value_is_not_a_noop() {
    assert_refires("INSERT INTO items (id, a, b) VALUES (1, 'x', COALESCE(now(), 'y'))");
}

#[test]
fn signed_volatile_insert_value_is_not_a_noop() {
    assert_refires("INSERT INTO items (id, a, b) VALUES (1, 'x', -now())");
}

#[test]
fn mixed_values_row_volatility_is_not_a_noop() {
    assert_refires("INSERT INTO items (id, a, b) VALUES (1, 'x', 'y'), (2, 'x', now())");
}

#[test]
fn before_insert_rewrite_is_not_a_noop() {
    let file =
        extract_sql_statement_facts(&sql("INSERT INTO items (id, a, b) VALUES (1, 'x', 'y')"));
    let triggers = extract_sql_statement_facts(
        "CREATE TRIGGER n BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION normalize();
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

#[test]
fn instead_of_insert_rewrite_is_not_a_noop() {
    let file =
        extract_sql_statement_facts(&sql("INSERT INTO items (id, a, b) VALUES (1, 'x', 'y')"));
    let mut triggers = extract_sql_statement_facts(
        "CREATE TRIGGER n INSTEAD OF INSERT ON items FOR EACH ROW EXECUTE FUNCTION normalize();
         CREATE TRIGGER t AFTER UPDATE OF a ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers;
    // sqlparser may map INSTEAD OF to BEFORE; pin the fact so the INSERT rewrite
    // arm is covered even when the SQL period token is normalized.
    for trigger in &mut triggers {
        if trigger.function.eq_ignore_ascii_case("normalize") {
            trigger.period = SqlTriggerPeriod::InsteadOf;
            trigger.for_each_row = true;
            trigger.events = vec![SqlTriggerEvent::Insert];
        }
    }
    let writes = [("normalize".to_string(), vec!["b".to_string()])];
    let replay_safe = ["normalize".to_string()];
    let found = judge_file(&file, &catalog(&triggers, &writes, &replay_safe));
    assert!(
        found.iter().any(|(_, message)| message.contains("audit")),
        "{found:?}"
    );
}

#[test]
fn placeholder_insert_value_still_noops() {
    for insert in [
        "INSERT INTO items (id, a, b) VALUES (1, 'x', sql_placeholder_1)",
        "INSERT INTO items (id, a, b) VALUES (1, 'x', -1)",
        "INSERT INTO items (id, a, b) VALUES (1, 'x', +1)",
        "INSERT INTO items (id, a, b) VALUES (1, 'x', 'epoch')",
    ] {
        let found = judge_file(
            &extract_sql_statement_facts(&sql(insert)),
            &catalog(&extract_sql_statement_facts(AFTER_A).triggers, &[], &[]),
        );
        assert!(found.is_empty(), "{insert} {found:?}");
    }
}
