use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    let mut path = rules_path("postgres-no-test-state-mutation", "fixture", "placeholder");
    path.pop();
    path
}

#[test]
fn postgres_no_test_state_mutation_check_json_snapshot() {
    let rule = "postgres-no-test-state-mutation";
    let root = fixture();
    let config = |name: &str| rules_path(rule, "configs", name);
    // include lists only the files each former check passed. Verdict ledger
    // fields are not in check --format json; those asserts stay in the unit tests.
    // advise/ignore do not emit unanalyzable-sql as a RuleFinding.
    assert_saved_fixture(
        "postgres_no_test_state_mutation",
        &[
            // Valid PostgreSQL multi-relation ANALYZE is clear; malformed SQL
            // stays unknown and the separate DROP TABLE remains a finding.
            (
                "analyze_relation_list",
                root.clone(),
                config("analyze-test.yml"),
            ),
            // Targets that must remain: DROP TABLE, CREATE TABLE, CREATE TABLE,
            // ALTER TABLE, CREATE VIEW, DROP VIEW, CREATE VIEW, ALTER SYSTEM,
            // session_replication_role, pg_sleep, pg_sleep, TRUNCATE.
            // Every file starts with tests/ or helpers/.
            (
                "executed_test_and_helper_sql",
                root.clone(),
                config("executed-with-helpers.yml"),
            ),
            // No finding file is helpers/reset.ts. Some target is CREATE TABLE.
            (
                "helper_roots_explicit",
                root.clone(),
                config("executed-without-helpers.yml"),
            ),
            // CLI honors the disable comments. Pre-suppression len 2 stays in
            // line_suppression_uses_the_shared_rule_directive.
            (
                "line_suppression",
                root.clone(),
                config("suppressed-test.yml"),
            ),
            // One unanalyzable-sql finding.
            (
                "unrecoverable_executor_sql",
                root.clone(),
                config("dynamic-test.yml"),
            ),
            // CREATE TABLE must remain.
            (
                "definite_variant_mutation",
                root.clone(),
                config("variants-test.yml"),
            ),
            // CREATE TABLE remains. unanalyzable-sql remains only for fail.
            ("advice_fail", root.clone(), config("advice-fail.yml")),
            ("advice_advise", root.clone(), config("advice-advise.yml")),
            ("advice_ignore", root.clone(), config("advice-ignore.yml")),
            // Every schema-object family and nested pg_sleep must remain:
            // CREATE/ALTER/DROP DATABASE, SCHEMA, TABLE, VIEW, TRIGGER,
            // FUNCTION, PROCEDURE, INDEX, TYPE, and pg_sleep.
            (
                "schema_object_families",
                root.clone(),
                config("operations-test.yml"),
            ),
            // Lines 8, 9, and 10, each target pg_sleep.
            (
                "explain_analyzed_sql",
                root.clone(),
                config("explain-test.yml"),
            ),
            // CREATE TABLE, session_replication_role, and pg_sleep remain.
            // unanalyzable-sql remains only for fail.
            ("prefix_fail", root.clone(), config("prefix-fail.yml")),
            ("prefix_advise", root.clone(), config("prefix-advise.yml")),
            ("prefix_ignore", root, config("prefix-ignore.yml")),
        ],
    );
}
