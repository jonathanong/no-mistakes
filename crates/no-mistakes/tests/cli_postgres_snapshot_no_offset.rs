use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

#[test]
fn postgres_no_offset_check_json_snapshot() {
    let rule = "postgres-no-offset";
    // `loose_selector_low` matches `loose_selector_max`.
    // `dynamic_unanalyzable_fail` matches `dynamic_unanalyzable_default` and
    // `dynamic_unanalyzable_fail_upper`. `effort_default` matches `effort_max`.
    // `namespace_methods_low` and `namespace_methods_max` have no findings; max
    // also reports `unprovenMemberQuerySql`. `crlf_ignore` is lines 4 and 8.
    // `crlf_low` adds unanalyzable at line 10. `crlf_max` reports offset#3 at
    // line 11 instead of that unanalyzable finding.
    let rows = [
        // One src/query.ts finding, target offset. The message contains
        // "do not use SQL OFFSET" and must not contain "optimizer fence".
        ("fail_offset", "fail", "embedded-query.yml"),
        (
            "standalone_table",
            "standalone-table",
            "standalone-query.yml",
        ),
        (
            "interpolated_offset",
            "fail-placeholder",
            "embedded-query.yml",
        ),
        ("limit_without_offset", "pass-limit", "embedded-query.yml"),
        ("offset_in_prose", "pass-prose", "embedded-query.yml"),
        ("disabled_next_line", "fail", "disabled-next-line.yml"),
        ("exclude_query", "fail", "exclude-query.yml"),
        ("include_query", "fail", "include-query.yml"),
        ("include_misses_file", "fail", "include-misses.yml"),
        ("other_executor_quiet", "fail", "other-executor.yml"),
        (
            "unanalyzable_ignored_embedded",
            "review-followups",
            "invalid-embedded-ignore.yml",
        ),
        // Only line 2, target unanalyzable. Unparseable static SQL stays quiet.
        (
            "unanalyzable_default_embedded",
            "review-followups",
            "invalid-embedded.yml",
        ),
        // Both the optimizer-fence message and "do not use SQL OFFSET" must remain.
        ("sql_views_offsets", "fail-sql", "sql-db-views.yml"),
        ("materialized_cte", "pass-sql", "sql-db-views.yml"),
        (
            "default_sql_include_unscanned",
            "pass-default",
            "sql-unscanned.yml",
        ),
        ("sql_outside_include", "skip-sql", "sql-outside.yml"),
        // Four findings must remain, including a fence and "do not use SQL OFFSET".
        ("edges_four_offsets", "edges", "sql-edges.yml"),
        (
            "embedded_offset_zero",
            "review-followups",
            "embedded-zero.yml",
        ),
        ("missing_sql_skipped", "fail-sql", "sql-missing.yml"),
        ("sql_suppressions", "suppress-sql", "sql-suppress.yml"),
        // Lines (5, unanalyzable), (10, unanalyzable), (20, offset),
        // (35, unanalyzable), and (38, unanalyzable) must remain.
        // Lines 15, 25, and the suppressed call stay quiet.
        // The first message starts `src/query.ts:5:` and contains `unanalyzableSql: ignore`.
        (
            "dynamic_unanalyzable_default",
            "unanalyzable",
            "embedded-query.yml",
        ),
        (
            "dynamic_unanalyzable_ignore",
            "unanalyzable",
            "dynamic-ignore.yml",
        ),
        (
            "dynamic_unanalyzable_fail",
            "unanalyzable",
            "dynamic-fail.yml",
        ),
        (
            "dynamic_unanalyzable_fail_upper",
            "unanalyzable",
            "dynamic-fail-upper.yml",
        ),
        // (src/client.mts, 6, unanalyzable) and (src/client.mts, 7, offset) must
        // remain. http-only.mts stays quiet.
        (
            "executor_provenance_fail",
            "executor-provenance",
            "provenance-fail.yml",
        ),
        (
            "executor_provenance_ignore",
            "executor-provenance",
            "provenance-ignore.yml",
        ),
        ("explain_offsets", "review-followups", "explain.yml"),
        // Duplicate executor name `last` stays canonicalized to lines 2 and 3.
        (
            "custom_executor_names",
            "review-followups",
            "custom-executors.yml",
        ),
        // Post-suppression lines 5, 8, 18, 18, and 21 remain. Lines 12 and 28 are suppressed.
        (
            "append_positions_suppressed",
            "review-followups",
            "append-positions.yml",
        ),
        // 21 findings must remain. Each file's targets are distinct, and each
        // file's first target is offset.
        (
            "dml_and_containers",
            "review-followups",
            "dml-containers.yml",
        ),
        ("keyword_locations", "review-followups", "locations.yml"),
        ("copy_data_locations", "review-followups", "copy-data.yml"),
        // Both findings stay on line 4.
        (
            "shared_declaration",
            "review-followups",
            "shared-declaration.yml",
        ),
        (
            "shared_declaration_suppressed",
            "review-followups",
            "shared-suppressed.yml",
        ),
        // Lines 2 and 3 remain. The first message contains "optimizer fence".
        ("lenient_ddl", "review-followups", "lenient.yml"),
        // Lines 4, 10, and 10 must remain.
        ("copy_header_edges", "review-followups", "copy-edge.yml"),
        (
            "copy_unterminated",
            "review-followups",
            "copy-unterminated.yml",
        ),
        ("stdin_string", "review-followups", "stdin-string.yml"),
        // Post-suppression lines 2, 8, 9, 11, 13, and 16 remain. Lines 19 and 22 are suppressed.
        (
            "source_map_suppressed",
            "review-followups",
            "source-map.yml",
        ),
        (
            "prepared_missing_sql",
            "review-followups",
            "review-missing-sql.yml",
        ),
        ("loose_selector_max", "effort", "loose-max.yml"),
        ("loose_selector_low", "effort", "loose-low.yml"),
        ("namespace_methods_max", "effort", "namespace-max.yml"),
        ("namespace_methods_low", "effort", "namespace-low.yml"),
        // ignore lines are 4 and 8 and are a subset of max. low's unanalyzable finding is line 10.
        ("crlf_max", "effort", "effort-crlf-max.yml"),
        ("crlf_low", "effort", "effort-crlf-low.yml"),
        ("crlf_ignore", "effort", "effort-crlf-ignore.yml"),
        // effort_low_ignore has 6 findings, is a subset of max, and contains src/query.ts line 8.
        // max has at least as many findings as effort_low.
        ("effort_max", "effort", "effort-max.yml"),
        ("effort_default", "effort", "effort-default.yml"),
        ("effort_low", "effort", "effort-low.yml"),
        ("effort_low_ignore", "effort", "effort-low-ignore.yml"),
        // Every rules-array finding file ends with .sql. Uncertainty is advisory.
        ("invalid_sql_advise", "effort", "effort-invalid-advise.yml"),
    ];
    let mut cases: Vec<(&str, PathBuf, PathBuf)> = rows
        .into_iter()
        .map(|(label, fixture, config)| {
            (
                label,
                rules_path(rule, "fixture", fixture),
                rules_path(rule, "configs", config),
            )
        })
        .collect();
    // Lines 7, 14, 22, 36, 51, 53 with targets offset, offset#2, offset#3,
    // offset#4, unanalyzable, unanalyzable. branched(true) makes the helper OFFSET real.
    cases.push((
        "helper_alias_appends",
        no_mistakes::codebase::ts_resolver::normalize_path(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/postgres-facts/embedded"),
        ),
        rules_path(rule, "configs", "helper-alias.yml"),
    ));
    assert_saved_fixture("postgres_no_offset", &cases);
}
