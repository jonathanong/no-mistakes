use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_sql_shape_policy_check_json_snapshot() {
    let rule = "postgres-sql-shape-policy";
    // Output cases formerly asserted by the rule's unit tests.
    // The snapshot keeps counts, messages, targets, files, lines, and clean reports.
    let cases = [
        // One correlated-exists-set-operation finding.
        (
            "correlated_exists_union",
            "fixture",
            "fail",
            "schema-001.yml",
        ),
        (
            "restricted_union_clean",
            "fixture",
            "pass",
            "schema-001.yml",
        ),
        (
            "uncorrelated_union_clean",
            "fixture",
            "pass",
            "uncorrelated_union_clean.yml",
        ),
        (
            "probe_union_clean",
            "fixture",
            "pass",
            "probe_union_clean.yml",
        ),
        (
            "derived_union_clean",
            "fixture",
            "pass",
            "derived_union_clean.yml",
        ),
        // The restricted correlated union is still one finding.
        (
            "restricted_correlated_union",
            "fixture",
            "fail",
            "restricted_correlated_union.yml",
        ),
        // CLI suppression is empty. The unit test still sees one finding before suppression.
        (
            "disabled_next_line",
            "fixture",
            "fail",
            "disabled_next_line.yml",
        ),
        // options.exclude drops sql/001.sql. Sibling SQL files in the fixture stay unscanned.
        (
            "option_exclude_drops_sql",
            "fixture",
            "fail",
            "option_exclude_drops_sql.yml",
        ),
        // options.include keeps the one correlated-exists-set-operation finding. Invalid include and exclude globs stay unit-test errors.
        (
            "option_include_keeps_sql",
            "fixture",
            "fail",
            "option_include_keeps_sql.yml",
        ),
        // A finding says the SQL is not statically recoverable.
        (
            "dynamic_sql_fail_closed",
            "fixture",
            "fail-dynamic",
            "dynamic_sql_fail_closed.yml",
        ),
        (
            "dynamic_sql_ignored",
            "fixture",
            "fail-dynamic",
            "dynamic_sql_ignored.yml",
        ),
        // A finding says the SQL could not be analyzed.
        (
            "unparseable_schema_sql",
            "fixture",
            "fail",
            "unparseable_schema_sql.yml",
        ),
        // Three findings, all correlated-exists-set-operation. The fragment-text assert stays in the unit test.
        (
            "embedded_builder_correlated_unions",
            "fixture",
            "fail-embedded-builders",
            "embedded_builder_correlated_unions.yml",
        ),
        (
            "uncorrelated_runtime_builder_clean",
            "fixture",
            "pass-embedded-builder",
            "embedded_builder_correlated_unions.yml",
        ),
        // The executed local builder is one finding, not two.
        (
            "executed_local_builder_once",
            "fixture",
            "fail-embedded-builder-executed",
            "embedded_builder_correlated_unions.yml",
        ),
        // One finding. The unit test still asserts the embedded fragment count is 1.
        (
            "fluent_builder_overlap_once",
            "fixture",
            "fail-embedded-builder-overlap",
            "embedded_builder_correlated_unions.yml",
        ),
        // One finding. The unit test still asserts the embedded fragment count is 1.
        (
            "multiline_builder_overlap_once",
            "fixture",
            "fail-embedded-builder-multiline-overlap",
            "embedded_builder_correlated_unions.yml",
        ),
        // One finding says builder SQL is not statically recoverable.
        (
            "opaque_builder_fail_closed",
            "fixture",
            "fail-embedded-builder-opaque",
            "embedded_builder_correlated_unions.yml",
        ),
        (
            "opaque_builder_ignored",
            "fixture",
            "fail-embedded-builder-opaque",
            "opaque_builder_ignored.yml",
        ),
        // One finding. The unrelated SQL statement type is the clean case below.
        (
            "imported_sql_statement_alias",
            "fixture",
            "fail-embedded-builder-sql-statement-alias",
            "embedded_builder_correlated_unions.yml",
        ),
        (
            "unrelated_sql_statement_type_clean",
            "fixture",
            "pass-unrelated-sql-statement-type",
            "embedded_builder_correlated_unions.yml",
        ),
        (
            "reassigned_builder_clean",
            "fixture",
            "pass-reassigned-sql-builder",
            "embedded_builder_correlated_unions.yml",
        ),
        // Sequential typed appends are one finding. Reassigned builders stay clean.
        (
            "sequential_typed_append",
            "fixture",
            "fail-sequential-sql-statement-append",
            "embedded_builder_correlated_unions.yml",
        ),
        (
            "non_sql_append_clean",
            "fixture",
            "pass-non-sql-append",
            "embedded_builder_correlated_unions.yml",
        ),
        // One finding says builder SQL is not statically recoverable.
        (
            "unparseable_builder_fail_closed",
            "fixture",
            "fail-embedded-builder-unparseable",
            "embedded_builder_correlated_unions.yml",
        ),
        (
            "unparseable_builder_ignored",
            "fixture",
            "fail-embedded-builder-unparseable",
            "opaque_builder_ignored.yml",
        ),
        // The full SELECT fragment is one finding and is not wrapped again.
        (
            "select_fragment_builder",
            "fixture",
            "fail-embedded-select-fragment",
            "embedded_builder_correlated_unions.yml",
        ),
        (
            "non_sql_string_returns_clean",
            "fixture",
            "pass-non-sql-string-returns",
            "non_sql_string_returns_clean.yml",
        ),
        // A finding says builder SQL is not statically recoverable. The `.mjs` return on line 5 must not be a finding.
        (
            "returned_sql_fragment_fail_closed",
            "fixture",
            "fail-returned-sql-fragment",
            "returned_sql_fragment_fail_closed.yml",
        ),
        // Lines 1, 3, and 4 are each both literal-limit and keyset-only-sweep. Line 1 keyset-only-sweep is the single lower-bound walk; a paired lower and upper bound is cursor_window_clean. deleted_at IS NULL is still selective here.
        (
            "literal_limits_and_keyset_walks",
            "fixture",
            "fail-bounded-iteration",
            "literal_limits_and_keyset_walks.yml",
        ),
        // Declaring deleted_at IS NULL non-selective adds the line 2 keyset-only-sweep. Lines 1, 3, and 4 stay both shapes.
        (
            "nonselective_deleted_at_walk",
            "fixture",
            "fail-bounded-iteration",
            "nonselective_deleted_at_walk.yml",
        ),
        (
            "bounded_work_selecting_clean",
            "fixture",
            "pass-bounded-iteration",
            "literal_limits_and_keyset_walks.yml",
        ),
        // id >= $1 AND id < $2 pages a range, so sql/002.sql is clean.
        (
            "cursor_window_clean",
            "fixture",
            "pass-bounded-iteration",
            "cursor_window_clean.yml",
        ),
        // keyset-only-sweep on lines 1, 2, and 4. WHERE TRUE, 1 = 1, a both-optional window, and ORDER BY random do not hide the walk. Line 3 stays clean.
        (
            "seeded_where_and_sample",
            "fixture",
            "fail-bounded-iteration",
            "seeded_where_and_sample.yml",
        ),
        // Without literal-limit or keyset-only-sweep, sql/001.sql is clean. The shapes are independent.
        (
            "bounded_shapes_off_by_default",
            "fixture",
            "fail-bounded-iteration",
            "bounded_shapes_off_by_default.yml",
        ),
        // literal-limit on lines 1, 3, and 4 only.
        (
            "literal_limit_only",
            "fixture",
            "fail-bounded-iteration",
            "literal_limit_only.yml",
        ),
        // keyset-only-sweep on lines 1, 3, and 4 only.
        (
            "keyset_only",
            "fixture",
            "fail-bounded-iteration",
            "keyset_only.yml",
        ),
        // Lines (5, keyset-only-sweep), (6, keyset-only-sweep), and (6, literal-limit).
        (
            "allowed_values_and_ignored_tables",
            "fixture",
            "allow-bounded-iteration",
            "allowed_values_and_ignored_tables.yml",
        ),
        // An empty allowedValues list allows no literal, not even 1. literal-limit is on lines 1, 2, and 6.
        (
            "empty_allowed_values",
            "fixture",
            "allow-bounded-iteration",
            "empty_allowed_values.yml",
        ),
        // keyset-only-sweep on lines 1, 2, and 3. A selective inner predicate is not a walk.
        (
            "quoted_dot_walks",
            "fixture",
            "quoted-bounded-iteration",
            "keyset_only.yml",
        ),
        // ignoreTables: [items] drops line 1 and keeps keyset-only-sweep on lines 2 and 3. It names work.items, not the one table "work.items".
        (
            "ignore_items_table_not_quoted_dot",
            "fixture",
            "quoted-bounded-iteration",
            "ignore_items_table_not_quoted_dot.yml",
        ),
        // Line 5 is both keyset-only-sweep and literal-limit.
        (
            "executor_sql_source_line",
            "fixture",
            "embedded-bounded-iteration",
            "executor_sql_source_line.yml",
        ),
        // template_interpolations_are_binds_for_the_cursor_and_the_batch_size and embedded_marker_identifiers_keep_their_source_provenance: only line 5 keyset-only-sweep. ${after} is the cursor and ${size} is the batch size.
        (
            "template_marker_keyset",
            "fixture",
            "embedded-bounded-iteration",
            "template_marker_keyset.yml",
        ),
        // keyset-only-sweep on lines 4 and 13. executed-fragment.mts is not scanned.
        (
            "builder_fragment_provenance",
            "fixture",
            "builder-fragment-provenance",
            "builder_fragment_provenance.yml",
        ),
        // CLI keeps literal-limit on line 3 and suppresses the disable-line FETCH on line 6. The unit test still asserts the raw finding on line 6.
        (
            "implicit_fetch_clause_suppressed",
            "fixture",
            "fail-bounded-iteration",
            "implicit_fetch_clause_suppressed.yml",
        ),
        // literal-limit on lines 1, 2, and 3.
        (
            "digit_separator_limits",
            "fixture",
            "digit-separators",
            "digit_separator_limits.yml",
        ),
        // keyset-only-sweep on lines 2, 5, and 6. Line 2 tells the user to list "Orders". Line 5 lists "work.Orders". Line 6 lists " orders ".
        (
            "ignore_tables_quoted_case",
            "fixture",
            "quoted-table-case",
            "ignore_tables_quoted_case.yml",
        ),
        // keyset-only-sweep on lines 3, 4, and 6 after ignoring "Orders" and "work.Orders".
        (
            "ignore_tables_quoted_identifiers",
            "fixture",
            "quoted-table-case",
            "ignore_tables_quoted_identifiers.yml",
        ),
        // literal-limit on lines 1, 2, and 3.
        (
            "radix_prefix_literal_limits",
            "fixture",
            "radix-prefix-separators",
            "radix_prefix_literal_limits.yml",
        ),
        // After suppression, literal-limit remains on lines 1 and 6. many.sql is not scanned.
        (
            "numeric_hex_literal_limits",
            "fixture",
            "radix-hex-literals",
            "radix_prefix_literal_limits.yml",
        ),
        // keyset-only-sweep on lines 2, 3, and 10.
        (
            "expanded_keysets",
            "fixture",
            "expanded-keysets",
            "expanded_keysets.yml",
        ),
        // keyset-only-sweep on lines 2 and 3.
        (
            "reordered_expanded_keysets",
            "fixture",
            "reordered-expanded-keysets",
            "expanded_keysets.yml",
        ),
        // keyset-only-sweep on lines 2 and 3.
        (
            "commuted_expanded_keysets",
            "fixture",
            "commuted-expanded-keysets",
            "expanded_keysets.yml",
        ),
        // keyset-only-sweep on lines 2, 8, 9, 15, 19, 21, and 23.
        (
            "casted_expanded_keysets",
            "fixture",
            "casted-expanded-keysets",
            "expanded_keysets.yml",
        ),
        // Only line 2 keyset-only-sweep. The predicate keeps its literal case.
        (
            "predicate_case_literal",
            "fixture",
            "predicate-case",
            "predicate_case_literal.yml",
        ),
        // keyset-only-sweep on lines 1 and 4. A bind-only guard does not hide the sweep.
        (
            "bind_only_guard",
            "fixture",
            "bind-guards",
            "keyset_only.yml",
        ),
    ]
    .map(|(label, kind, fixture, config)| {
        (
            label,
            rules_path(rule, kind, fixture),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_sql_shape_policy", &cases);
}
