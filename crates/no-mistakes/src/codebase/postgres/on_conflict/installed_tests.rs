use super::{installed, judge_file, Catalog};
use crate::codebase::postgres::statement_facts::SqlTriggerFact;
use crate::codebase::postgres::statements::extract_sql_statement_facts;
use std::path::PathBuf;

fn catalog<'a>(triggers: &'a [SqlTriggerFact]) -> Catalog<'a> {
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

fn positioned(sql: &str, path: &str) -> SqlTriggerFact {
    let mut fact = extract_sql_statement_facts(sql).triggers.remove(0);
    fact.source_path = PathBuf::from(path);
    fact
}

#[test]
fn unpositioned_drops_do_not_hide_creates() {
    let created = extract_sql_statement_facts(
        "CREATE TRIGGER items_before_insert BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();",
    )
    .triggers
    .remove(0);
    let mut dropped = created.clone();
    dropped.removed = true;
    let file = extract_sql_statement_facts(
        "INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;",
    );
    let triggers = [dropped, created];
    let found = judge_file(&file, &catalog(&triggers));
    assert!(
        found
            .iter()
            .any(|(_, message)| message.contains("fires before conflict checks")),
        "{found:?}"
    );
}

#[test]
fn drop_without_a_table_matches_the_trigger_name() {
    let created = positioned(
        "CREATE TRIGGER items_before_insert BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();",
        "sql/001.sql",
    );
    let mut dropped = created.clone();
    dropped.removed = true;
    dropped.table.clear();
    dropped.events.clear();
    dropped.source_path = PathBuf::from("sql/002.sql");
    let mut file = extract_sql_statement_facts(
        "INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;",
    );
    file.path = PathBuf::from("sql/003.sql");
    let triggers = [created, dropped];
    let found = installed::at(&triggers, &file.path, file.inserts[0].line);
    assert!(found.is_empty(), "{found:?}");
}
