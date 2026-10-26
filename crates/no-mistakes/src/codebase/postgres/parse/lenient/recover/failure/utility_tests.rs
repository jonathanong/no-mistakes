use crate::codebase::postgres::extract_migration_facts;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/regress-utilities")
            .join(name),
    )
    .unwrap()
}

#[test]
fn postgres_regress_utilities_parse_or_retain_recognized_unsupported_diagnostics() {
    for name in ["valid.sql", "variants.sql"] {
        for sql in fixture(name).lines() {
            let facts = extract_migration_facts(sql);
            assert!(
                facts
                    .unanalyzable_statements
                    .iter()
                    .all(|item| item.valid_unsupported),
                "{sql}: {:?}",
                facts.unanalyzable_statements
            );
        }
    }
}

#[test]
fn malformed_utility_near_misses_remain_failures() {
    for sql in fixture("malformed.sql").lines() {
        let facts = extract_migration_facts(sql);
        assert!(!facts.unanalyzable_statements.is_empty(), "{sql}");
        assert!(
            facts
                .unanalyzable_statements
                .iter()
                .all(|item| !item.valid_unsupported),
            "{sql}"
        );
    }
}

#[test]
fn rule_action_semicolons_do_not_create_orphan_failures_or_hide_later_sql() {
    let facts = extract_migration_facts(&fixture("rule-actions.sql"));
    assert_eq!(
        facts.unanalyzable_statements.len(),
        2,
        "{:?}",
        facts.unanalyzable_statements
    );
    assert!(facts.unanalyzable_statements[0]
        .head
        .starts_with("CREATE RULE"));
    assert!(facts.unanalyzable_statements[1]
        .head
        .starts_with("CREATE OR REPLACE RULE"));
    assert_eq!(facts.tables.len(), 1);
    assert_eq!(facts.tables[0].table_name, "after_rule");
}
