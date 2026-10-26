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

fn fixture_facts(name: &str) -> crate::codebase::postgres::SqlStatementFileFacts {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/check-runner/postgres-parse-failure-locations/fixture")
        .join(name);
    extract_sql_statement_facts(&std::fs::read_to_string(path).unwrap())
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
fn array_target_conflicts_survive_surrounding_statements() {
    let context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/array-conflict-context.sql"
    ));
    // Earlier unrecovered inserts shift ordinal-based source locations in the complete file.
    let full = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/arrays-regress-full.sql"
    ));
    for (sql, lines) in [(context, [19, 22]), (full, [427, 430])] {
        let file = extract_sql_statement_facts(sql);
        for line in lines {
            let insert = file
                .inserts
                .iter()
                .find(|insert| insert.line == line)
                .unwrap_or_else(|| panic!("line {line}: {:?}", file.inserts));
            assert!(insert.on_conflict.is_some(), "line {line}: {insert:?}");
            assert!(file.writes.iter().any(|write| write.line == line));
        }
        let unguarded = if lines[0] == 427 { 426 } else { 18 };
        assert!(file
            .inserts
            .iter()
            .any(|insert| { insert.line == unguarded && insert.on_conflict.is_none() }));
        let found = judge_file(&file, &catalog(&[], &[], &[], &[]));
        assert!(found.iter().any(|(line, message)| {
            *line == unguarded && message.contains("must include ON CONFLICT")
        }));
        assert!(!found.iter().any(|(line, message)| {
            lines.contains(line) && message.contains("must include ON CONFLICT")
        }));
    }
}

#[test]
fn unrecovered_insert_does_not_shift_later_conflicts_or_plain_insert() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/array-conflict-gap.sql"
    ));
    let file = extract_sql_statement_facts(sql);
    assert!(file.parse_failed);
    assert!(file.parse_failures.iter().any(|failure| failure.line == 2));
    for line in [3, 4] {
        let insert = file
            .inserts
            .iter()
            .find(|insert| insert.line == line)
            .unwrap();
        assert!(insert.on_conflict.is_some(), "line {line}: {insert:?}");
    }
    let plain = file.inserts.iter().find(|insert| insert.line == 6).unwrap();
    assert!(plain.on_conflict.is_none());
    let override_insert = file.inserts.iter().find(|insert| insert.line == 7).unwrap();
    assert!(override_insert.assignments.is_empty());
    let found = judge_file(&file, &catalog(&[], &[], &[], &[]));
    assert!(found
        .iter()
        .any(|(line, message)| *line == 6 && message.contains("ON CONFLICT")));
    assert!(!found.iter().any(
        |(line, message)| [3, 4].contains(line) && message.contains("must include ON CONFLICT")
    ));
}

#[test]
fn recovered_same_line_inserts_use_original_unicode_columns() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/array-conflict-same-line.sql"
    ));
    let file = extract_sql_statement_facts(sql);
    let on_line = file
        .inserts
        .iter()
        .filter(|insert| insert.line == 2)
        .map(|insert| insert.on_conflict.is_some())
        .collect::<Vec<_>>();
    assert_eq!(on_line, [true, false]);
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
    let catalog = catalog(&[], &[], &[], &[]);
    let message = "INSERT could not be proven replay-safe".to_string();
    // A valid sibling cannot alter the malformed statement's diagnostic.
    assert_eq!(
        judge_file(&fixture_facts("failed-insert.sql"), &catalog),
        [(2, message.clone())]
    );
    assert_eq!(
        judge_file(&fixture_facts("safe-and-failed-insert.sql"), &catalog),
        [(3, message)]
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
    let found = judge_file(
        &fixture_facts("two-failed-inserts.sql"),
        &catalog(&[], &[], &[], &[]),
    );
    assert_eq!(
        found,
        [
            (2, "INSERT could not be proven replay-safe".to_string()),
            (4, "INSERT could not be proven replay-safe".to_string())
        ]
    );
}

#[test]
fn one_unparseable_multi_insert_fragment_still_requires_hoisting() {
    let found = judge_file(
        &fixture_facts("multi-insert-fragment.sql"),
        &catalog(&[], &[], &[], &[]),
    );
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, 2);
    assert!(found[0].1.contains("more than one INSERT"));
}

#[test]
fn older_statement_projections_keep_the_legacy_insert_fallback() {
    let catalog = catalog(&[], &[], &[], &[]);
    for (name, expected) in [
        ("multi-insert-fragment.sql", 1),
        ("failed-insert.sql", 1),
        ("guarded-failed-insert.sql", 0),
    ] {
        let mut facts = fixture_facts(name);
        assert!(facts.parse_failed, "{name}");
        // Simulate a caller supplying facts without the new occurrence field.
        facts.parse_failures.clear();
        let findings = judge_file(&facts, &catalog);
        assert_eq!(findings.len(), expected, "{name}: {findings:?}");
        if name == "multi-insert-fragment.sql" {
            assert!(findings[0].1.contains("more than one INSERT"));
        }
    }
}

#[test]
fn unparseable_single_insert_with_not_exists_is_safe() {
    let sql =
        "INSERT INTO items (id) SELECT 1 WHERE NOT EXISTS (SELECT 1 FROM items WHERE id = 1) !!!";
    assert!(messages(sql).is_empty() || extract_sql_statement_facts(sql).has_top_level_not_exists);
}

include!("tests/catalog_and_triggers.rs");
include!("tests/assignments.rs");
