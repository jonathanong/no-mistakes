use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_explicit_columns_check_json_snapshot() {
    let rule = "postgres-explicit-columns";
    // Counts, messages, lines, targets, emptiness, and option effects from the
    // rule's check_with_files tests. Suppression cases with directives are the
    // CLI outcome; sibling fixtures drop the directive and keep the line.
    let cases = [
        ("reported", "reported", "relations.yml"),
        ("standalone_table", "standalone", "relations.yml"),
        ("table_set_arms", "arms", "relations.yml"),
        ("valid_examples", "pass", "relations.yml"),
        // Messages must not contain view_public_tags.
        ("max_columns_zero", "max-zero", "max-zero.yml"),
        ("joins_and_qualified", "wide", "relations.yml"),
        (
            "cte_alias_and_json_upper",
            "quiet-wrappers",
            "relations.yml",
        ),
        ("returning_disabled", "returning-only", "returning-off.yml"),
        (
            "whole_row_functions_cleared",
            "json-upper",
            "whole-row-off.yml",
        ),
        ("default_sql_include", "edges", "catalog-only.yml"),
        ("suppressed_stars", "suppress", "relations.yml"),
        ("unsuppressed_stars", "suppress-raw", "relations.yml"),
        // imported-source.mts:32 must be present.
        ("helper_literals", "helper-raw", "helper.yml"),
        // imported-source.mts:32 must be absent.
        (
            "helper_literals_suppressed",
            "helper-suppressed",
            "helper.yml",
        ),
        // Findings must not name src/imported-.
        ("helper_without_caller", "helper-no-caller", "helper.yml"),
        ("projection_lines", "lines", "deferred.yml"),
        ("qualified_targets", "qualified", "deferred.yml"),
        ("cte_names", "ctes", "deferred.yml"),
        (
            "qualified_relation_list",
            "qualified",
            "deferred-listed.yml",
        ),
        ("dml_cte_returning", "dml-ctes", "deferred.yml"),
        (
            "dml_cte_returning_disabled",
            "dml-ctes",
            "deferred-returning-off.yml",
        ),
        ("returning_peers", "peers", "deferred.yml"),
        (
            "whole_row_expressions",
            "functions",
            "deferred-whole-row-off.yml",
        ),
        ("whole_row_functions_allowed", "functions", "deferred.yml"),
        ("values_read", "values", "deferred.yml"),
        ("dynamic_and_parse_failures", "queries", "deferred-db.yml"),
        (
            "dynamic_failures_ignored",
            "queries",
            "deferred-db-ignore.yml",
        ),
        ("invalid_sql", "invalid", "deferred.yml"),
        ("invalid_sql_ignored", "invalid", "deferred-ignore.yml"),
        // Messages must not contain view_public_tags.
        ("known_view_unlisted", "view", "deferred-view-max.yml"),
        ("known_view_listed", "view", "deferred-view-listed.yml"),
        ("insert_aliases", "insert-alias", "deferred.yml"),
        ("function_lines", "function-lines-raw", "lenient-lines.yml"),
        (
            "function_lines_suppressed",
            "function-lines",
            "lenient-lines.yml",
        ),
        ("lenient_lines", "lenient-lines-raw", "lenient-lines.yml"),
        (
            "lenient_lines_suppressed",
            "lenient-lines",
            "lenient-lines.yml",
        ),
        (
            "lenient_returning_lines",
            "lenient-returning-raw",
            "lenient-returning.yml",
        ),
        (
            "lenient_returning_suppressed",
            "lenient-returning",
            "lenient-returning.yml",
        ),
        ("span_suppress_sql", "suppress-sql-raw", "deferred-db.yml"),
        (
            "span_suppress_sql_hidden",
            "suppress-sql",
            "deferred-db.yml",
        ),
        ("span_suppress_ts", "suppress-ts-raw", "deferred-db.yml"),
        ("span_suppress_ts_hidden", "suppress-ts", "deferred-db.yml"),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(rule, "fixture", fixture),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_explicit_columns", &cases);
}
