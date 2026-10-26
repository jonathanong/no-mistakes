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
    let found = installed::InstalledTriggers::new(&triggers)
        .at(&file.path, file.inserts[0].line, "items")
        .cloned()
        .collect::<Vec<_>>();
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn unpositioned_create_applies_beside_positioned_events() {
    let created = positioned(
        "CREATE TRIGGER items_before_insert BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();",
        "sql/001.sql",
    );
    let mut dropped = created.clone();
    dropped.removed = true;
    dropped.source_path = PathBuf::from("sql/002.sql");
    let mut always = created.clone();
    always.source_path.clear();
    always.name = "other_trigger".into();
    let mut ignored = always.clone();
    ignored.removed = true;
    ignored.name = "gone".into();
    let mut file = extract_sql_statement_facts(
        "INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;",
    );
    file.path = PathBuf::from("sql/003.sql");
    let triggers = [created, dropped, always, ignored];
    let found = installed::InstalledTriggers::new(&triggers)
        .at(&file.path, file.inserts[0].line, "items")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        found
            .iter()
            .map(|trigger| trigger.name.as_str())
            .collect::<Vec<_>>(),
        ["other_trigger"],
        "{found:?}"
    );
}

#[test]
fn prepared_timeline_replays_events_once_across_repeated_insert_lookups() {
    let triggers = [positioned(
        "CREATE TRIGGER items_before_insert BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();",
        "sql/001.sql",
    )];
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let prepared = crate::diagnostics::with_observer(Some(observer.clone()), || {
        installed::InstalledTriggers::new(&triggers)
    });
    for _ in 0..100 {
        let found = prepared
            .at(std::path::Path::new("app/q.ts"), 1, "items")
            .next()
            .unwrap();
        assert!(std::ptr::eq(found, &triggers[0]));
    }
    assert_eq!(
        observer.snapshot().work["postgres.trigger.timeline_events"],
        1
    );
}

#[test]
fn public_judge_prepares_once_and_disabled_checks_prepare_nothing() {
    let triggers = [positioned(
        "CREATE TRIGGER items_before_insert BEFORE INSERT ON items FOR EACH ROW EXECUTE FUNCTION audit();",
        "sql/001.sql",
    )];
    let mut file = extract_sql_statement_facts(
        "INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING;",
    );
    file.path = PathBuf::from("app/q.ts");
    file.inserts = vec![file.inserts[0].clone(); 100];
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let findings = crate::diagnostics::with_observer(Some(observer.clone()), || {
        judge_file(&file, &catalog(&triggers))
    });
    assert_eq!(findings.len(), 100);
    assert_eq!(
        observer.snapshot().work["postgres.trigger.timeline_builds"],
        1
    );
    let disabled = Catalog {
        check_triggers: false,
        ..catalog(&triggers)
    };
    assert!(
        crate::diagnostics::with_observer(Some(observer.clone()), || judge_file(&file, &disabled))
            .is_empty()
    );
    assert_eq!(
        observer.snapshot().work["postgres.trigger.timeline_builds"],
        1
    );
}
