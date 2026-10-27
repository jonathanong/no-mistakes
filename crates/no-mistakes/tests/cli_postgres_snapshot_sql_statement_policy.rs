use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_sql_statement_policy_check_json_snapshot() {
    let rule = "postgres-sql-statement-policy";
    // Output cases formerly asserted by the rule's unit tests.
    // The snapshot keeps counts, messages, targets, files, lines, and clean reports.
    let cases = [
        // flags_create_table: one CREATE TABLE finding.
        ("create_table", "fixture", "fail", "sql-001.yml"),
        // Targets are exactly CREATE TABLE, ALTER TABLE, CREATE INDEX, CREATE VIEW, TRUNCATE, DROP INDEX, DROP VIEW.
        (
            "default_banned_kinds",
            "fixture",
            "fail-kinds",
            "sql-001.yml",
        ),
        // One CREATE TABLE finding inside the DO block.
        (
            "create_table_inside_do_block",
            "fixture",
            "fail-do-block",
            "sql-001.yml",
        ),
        ("inserts_and_functions", "fixture", "pass", "sql-001.yml"),
        // Only line 5 CREATE TABLE. Other CREATE kinds do not inherit the table policy.
        (
            "comments_and_other_create_kinds",
            "fixture",
            "pass-postgres-regress",
            "sql-001.yml",
        ),
        // bannedStatements: [TRUNCATE] leaves a single TRUNCATE finding.
        (
            "custom_banned_truncate",
            "fixture",
            "fail-kinds",
            "custom_banned_truncate.yml",
        ),
        // CLI suppression of sql/disabled.sql is empty. The unit test still checks the pre-suppression finding.
        (
            "disabled_next_line",
            "fixture",
            "fail",
            "disabled_next_line.yml",
        ),
        // excluded.test.ts and runtime.ts were passed but are outside include/exclude, so they must not appear. Findings are policy.test.ts ALTER TABLE line 2, CREATE VIEW line 4, TRUNCATE line 6.
        (
            "embedded_executor_scope",
            "fixture",
            "embedded",
            "embedded_executor_scope.yml",
        ),
        // unanalyzableSql: ignore reports nothing. invalid unanalyzableSql remains a unit-test error.
        (
            "dynamic_unparseable_ignored",
            "fixture",
            "embedded",
            "dynamic_unparseable_ignored.yml",
        ),
        // Lines 2 and 3 are unanalyzable-sql.
        (
            "dynamic_unparseable_fail",
            "fixture",
            "embedded",
            "dynamic_unparseable_fail.yml",
        ),
        // CLI suppression is empty. The unit test still sees 3 findings before suppress_rule_findings_with_sources.
        (
            "embedded_ts_suppression",
            "fixture",
            "embedded",
            "embedded_ts_suppression.yml",
        ),
        // CREATE TABLE on line 4 and ALTER TABLE on line 5.
        (
            "embedded_static_routines",
            "fixture",
            "embedded",
            "embedded_static_routines.yml",
        ),
        // A sql-only policy does not report embedded TypeScript. Plan flags and the zero physical read stay in the unit test.
        (
            "legacy_sql_only_skips_ts",
            "fixture",
            "embedded",
            "legacy_sql_only_skips_ts.yml",
        ),
        // Only line 6 TRUNCATE remains.
        (
            "embedded_custom_banned_truncate",
            "fixture",
            "embedded",
            "embedded_custom_banned_truncate.yml",
        ),
        // No finding message says it could not be analyzed. Definite targets are CREATE DATABASE, DROP DATABASE, ALTER DATABASE, ALTER SYSTEM, CREATE SCHEMA, ALTER SCHEMA, DROP SCHEMA, CREATE TRIGGER, DROP TRIGGER, CREATE FUNCTION, CREATE PROCEDURE, DROP FUNCTION, DROP PROCEDURE, DROP TABLE, CREATE TYPE, DROP TYPE, CREATE TABLE, ALTER TABLE, CREATE TRIGGER.
        (
            "expanded_kinds",
            "fixture",
            "expanded",
            "expanded_kinds.yml",
        ),
        // setting:* findings are lines 1 through 9. Non-setting literals stay inert.
        (
            "setting_commands",
            "fixture",
            "expanded",
            "setting_commands.yml",
        ),
        // Lines and targets: (2, ALTER DATABASE), (3, setting:session_replication_role), (4, setting:session_replication_role), (5, CREATE DATABASE), (7, CREATE FUNCTION), (8, CREATE SCHEMA), (9, setting:session_replication_role), (10, DROP SCHEMA), (12, CREATE PROCEDURE), (13, ALTER SYSTEM), (14, DROP DATABASE).
        (
            "routine_and_execute_settings",
            "fixture",
            "expanded",
            "routine_and_execute_settings.yml",
        ),
        // (2, CREATE DATABASE), (3, setting:session_replication_role), (4, setting:session_replication_role), (5, ALTER SYSTEM), (6, CREATE PROCEDURE).
        (
            "embedded_utility_grammar",
            "fixture",
            "expanded",
            "embedded_utility_grammar.yml",
        ),
        // CLI suppression is empty. The unit test still sees 2 findings before suppression.
        (
            "disabled_sql_settings",
            "fixture",
            "expanded",
            "disabled_sql_settings.yml",
        ),
        // Procedural prefixes stop before CASE branches. Lines: (1, CREATE FUNCTION), (2, ALTER DATABASE), (3, ALTER SYSTEM), (5, CREATE PROCEDURE), (6, setting:session_replication_role), (9, setting:session_replication_role), (12, setting:session_replication_role).
        (
            "procedural_prefixes",
            "fixture",
            "expanded",
            "procedural_prefixes.yml",
        ),
        // Unanalyzable targets stay in rules: lines 3, 21, and 39 CREATE DATABASE scratch TEMPLATE template0; lines 5, 23, and 41 ALTER DATABASE app SET statement_timeout = '1s'; lines 6, 24, and 42 ALTER SYSTEM SET statement_timeout = '2s'; line 13 CREATE OR REPLACE PROCEDURE empty_proc() LANGUAGE. 49 definite findings remain. CREATE FUNCTION appears 4 times. Each other kind from CREATE DATABASE through DROP TYPE appears 3 times. Later ALTER object kinds are absent.
        (
            "all_contexts_reported",
            "fixture",
            "expanded",
            "all_contexts_reported.yml",
        ),
        // setting:session_replication_role is on lines 2, 3, and 6.
        (
            "setting_contexts",
            "fixture",
            "expanded",
            "setting_contexts.yml",
        ),
        // One ALTER TABLE finding.
        (
            "disable_trigger_is_alter_table",
            "fixture",
            "expanded",
            "disable_trigger_is_alter_table.yml",
        ),
        // Only lines 1 and 2. Escaped and dollar-quoted setting names are inert.
        (
            "setting_literals",
            "fixture",
            "expanded",
            "setting_literals.yml",
        ),
        // Lines 1 through 4 are setting:timezone. TIME ZONE uses the timezone parameter name.
        (
            "time_zone_alias",
            "fixture",
            "expanded",
            "time_zone_alias.yml",
        ),
        // Post-suppression lines: failures.sql 3 and 6, one-suppressed.sql 6, second-suppressed.sql 3. both-suppressed.sql is absent.
        (
            "raw_parse_failure_sites",
            "fixture",
            "parse-failure-raw-sites",
            "parse-failure-raw.yml",
        ),
        // Post-suppression unanalyzable-sql: failures.test.ts lines 4 and 7, one-suppressed.test.ts line 7, second-suppressed.test.ts line 4. both-suppressed.test.ts is absent.
        (
            "embedded_parse_failure_sites",
            "fixture",
            "parse-failure-sites",
            "parse-failure-embedded.yml",
        ),
    ]
    .map(|(label, kind, fixture, config)| {
        (
            label,
            rules_path(rule, kind, fixture),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_sql_statement_policy", &cases);
}
