use super::{judge_file, judge_subcheck, Catalog};
use crate::codebase::postgres::statement_facts::SqlTriggerFact;
use crate::codebase::postgres::statements::extract_sql_statement_facts;
use crate::codebase::postgres::types::{
    SqlColumnMetadata, SqlCreateTableMetadata, SqlSchemaFileFacts,
};
use crate::codebase::postgres::SqlVerdictStatus;

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
fn patch_gap_unknown_enabled_subcheck_abstains() {
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO UPDATE SET id = EXCLUDED.id;",
    );
    let judgment = judge_subcheck("not-a-subcheck", &file, &catalog(&[], &[], &[], &[]));
    assert_eq!(judgment.status, SqlVerdictStatus::Abstain);
    assert_eq!(judgment.reason, Some("unknown-enabled-subcheck"));
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

include!("tests/catalog_and_triggers.rs");
include!("tests/assignments.rs");
