use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_sql_shape_policy_bounded_check_json_snapshot() {
    let rule = "postgres-sql-shape-policy";
    // Output cases formerly asserted by the rule's unit tests.
    // The snapshot keeps counts, messages, targets, files, lines, and clean reports.
    let cases = [
        // keyset-only-sweep on every line from 2 through 11.
        (
            "directional_keysets",
            "fixture",
            "directional-keysets",
            "expanded_keysets.yml",
        ),
        // keyset-only-sweep on lines 2, 3, 4, 5, and 6. Negated comparisons are cursors; opposite bounds form windows and are absent.
        (
            "negated_keyset_pages",
            "fixture",
            "negated-keysets",
            "expanded_keysets.yml",
        ),
        // keyset-only-sweep on lines 2, 3, 5, and 6.
        (
            "negated_keyset_parity",
            "fixture",
            "negated-keysets",
            "negated_keyset_parity.yml",
        ),
        // keyset-only-sweep on lines 2 through 9. recovered.sql is not scanned.
        (
            "cast_context_keysets",
            "fixture",
            "cast-context-keysets",
            "expanded_keysets.yml",
        ),
        // CLI suppresses the disable-next-line finding that the unit test still expects as raw keyset-only-sweep on line 28. qualified-zero.sql is not scanned.
        (
            "semantic_zero_pages",
            "fixture",
            "semantic-zero",
            "semantic_zero_pages.yml",
        ),
        // keyset-only-sweep on lines 2, 3, 6, and 7. Distinct pages are not findings.
        (
            "select_all_keyset_pages",
            "fixture",
            "select-all",
            "expanded_keysets.yml",
        ),
        // keyset-only-sweep on lines 2, 5, 8, and 11. Escaped delimiters stay selective.
        (
            "predicate_escaping",
            "fixture",
            "predicate-escaping",
            "predicate_escaping.yml",
        ),
        // keyset-only-sweep on lines 2, 4, 6, 8, and 10. Equivalent spellings match without colliding.
        (
            "predicate_normalization",
            "fixture",
            "predicate-escaping",
            "predicate_normalization.yml",
        ),
        // not-in-subquery and count-for-existence stay off by default, so this file is clean. The default shape flags stay a compile_options unit test.
        (
            "default_shapes_clean",
            "fixture",
            "pass-default-shapes",
            "bounded_shapes_off_by_default.yml",
        ),
        // A finding contains 'NOT IN (SELECT …) returns no rows when the subquery yields a NULL'.
        (
            "not_in_subquery_message",
            "fixture",
            "fail-shapes",
            "not_in_subquery_message.yml",
        ),
        // A finding contains 'COUNT(*) compared with 0/1 counts every matching row'.
        (
            "count_for_existence_message",
            "fixture",
            "fail-shapes",
            "count_for_existence_message.yml",
        ),
        (
            "pass_shapes_base",
            "fixture",
            "pass-shapes",
            "pass_shapes_base.yml",
        ),
        (
            "pass_shapes_list",
            "fixture",
            "pass-shapes",
            "pass_shapes_list.yml",
        ),
        (
            "pass_shapes_having",
            "fixture",
            "pass-shapes",
            "pass_shapes_having.yml",
        ),
        (
            "pass_shapes_threshold",
            "fixture",
            "pass-shapes",
            "pass_shapes_threshold.yml",
        ),
        (
            "pass_shapes_plain_count",
            "fixture",
            "pass-shapes",
            "pass_shapes_plain_count.yml",
        ),
        // Identical shapes on one line are one finding.
        (
            "identical_shapes_one_line",
            "fixture",
            "fail-shapes",
            "identical_shapes_one_line.yml",
        ),
        // CLI suppression is empty. The unit test still sees a finding before suppression.
        (
            "suppress_next_line",
            "fixture",
            "suppress-shapes",
            "suppress_next_line.yml",
        ),
        // CLI suppression is empty. The unit test still sees a finding before suppression.
        (
            "suppress_line",
            "fixture",
            "suppress-shapes",
            "suppress_line.yml",
        ),
        // CLI suppression is empty. The unit test still sees a finding before suppression.
        (
            "suppress_file",
            "fixture",
            "suppress-shapes",
            "suppress_file.yml",
        ),
        // Disabling correlated-exists-set-operation drops that finding. not-in-subquery alone does not flag this file.
        (
            "exists_shape_disabled_clean",
            "fixture",
            "fail",
            "exists_shape_disabled_clean.yml",
        ),
        // Lines 3 and 4.
        (
            "review_having",
            "fixture",
            "review-followups",
            "pass_shapes_having.yml",
        ),
        // Line 2.
        (
            "review_comments",
            "fixture",
            "review-followups",
            "review_comments.yml",
        ),
        // Lines 2 and 3.
        (
            "review_unicode",
            "fixture",
            "review-followups",
            "review_unicode.yml",
        ),
        // Lines 2 and 4.
        (
            "review_repeated",
            "fixture",
            "review-followups",
            "review_repeated.yml",
        ),
        // Lines 1 and 2.
        (
            "review_casts",
            "fixture",
            "review-followups",
            "review_casts.yml",
        ),
        // Line 2.
        (
            "review_filter",
            "fixture",
            "review-followups",
            "review_filter.yml",
        ),
        // Lines 2 and 4.
        (
            "review_mutations",
            "fixture",
            "review-followups",
            "review_mutations.yml",
        ),
        // not-count.sql is clean.
        (
            "review_not_count",
            "fixture",
            "review-followups",
            "review_not_count.yml",
        ),
        // Lines 1 through 6. The message contains 'use NOT EXISTS' on lines 2 through 5 only.
        (
            "review_polarity",
            "fixture",
            "review-followups",
            "review_polarity.yml",
        ),
        // Lines 1 and 2.
        (
            "review_cte",
            "fixture",
            "review-followups",
            "review_cte.yml",
        ),
        // Lines 2 and 4.
        (
            "review_operators",
            "fixture",
            "review-followups",
            "review_operators.yml",
        ),
        // Lines 1 and 3. The malformed COUNT at line 3 keeps its own uncertainty location.
        (
            "review_count_edges",
            "fixture",
            "review-followups",
            "review_count_edges.yml",
        ),
        // CLI keeps lines 2, 6, 8, and 10. The unit test still sees raw line 12 before suppression.
        (
            "source_map_suppressed",
            "fixture",
            "review-followups",
            "source_map_suppressed.yml",
        ),
        // CLI suppression is empty. The unit test still sees 2 findings before suppression.
        (
            "review_suppress_sql",
            "fixture",
            "review-followups",
            "review_suppress_sql.yml",
        ),
        // Rebased embedded findings are lines 5 and 6.
        (
            "embedded_shape_locations",
            "fixture",
            "review-followups",
            "embedded_shape_locations.yml",
        ),
        // Every finding target is not-in-subquery.
        (
            "unanalyzable_not_in_subquery",
            "fixture",
            "fail-dynamic",
            "unanalyzable_not_in_subquery.yml",
        ),
        // Every finding target is count-for-existence.
        (
            "unanalyzable_count_for_existence",
            "fixture",
            "fail-dynamic",
            "unanalyzable_count_for_existence.yml",
        ),
        // Lines 1, 2, 3, and 5. The message contains 'use NOT EXISTS' on lines 2 and 3 only.
        (
            "negated_count_existence",
            "fixture",
            "review-followups",
            "negated_count_existence.yml",
        ),
        // Lines 1, 2, and 3. Quoted lowercase count keeps the builtin schema and case restrictions.
        (
            "quoted_lowercase_count",
            "fixture",
            "review-followups",
            "quoted_lowercase_count.yml",
        ),
        // CLI keeps line 4. The unit test still sees raw lines 3 and 4, plus the private filter-fact counts.
        (
            "lenient_locations_suppressed",
            "fixture",
            "review-followups",
            "lenient_locations_suppressed.yml",
        ),
        // (not-in-subquery, 2), (not-in-subquery, 3), (count-for-existence, 4), (not-in-subquery, 5), (not-in-subquery, 6), (not-in-subquery, 8). Each shape is collected once.
        (
            "merge_shapes",
            "fixture",
            "review-followups",
            "merge_shapes.yml",
        ),
        // count-for-existence on lines 2 and 7.
        (
            "merge_table_function",
            "fixture",
            "review-followups",
            "merge_table_function.yml",
        ),
        // (not-in-subquery, 5) and (count-for-existence, 10).
        (
            "merge_action",
            "fixture",
            "review-followups",
            "merge_action.yml",
        ),
        // Lines 1 and 4.
        (
            "merge_with_cte",
            "fixture",
            "review-followups",
            "merge_with_cte.yml",
        ),
        // (not-in-subquery, 3) and (count-for-existence, 5).
        (
            "conflict_update_where",
            "fixture",
            "review-followups",
            "conflict_update_where.yml",
        ),
        // Nullable COUNT is skipped so null semantics stay intact.
        (
            "nullable_count_clean",
            "fixture",
            "review-followups",
            "nullable_count_clean.yml",
        ),
        // CLI suppression is empty. The unit test still sees raw line 3 before suppression.
        (
            "interpolation_start_suppressed",
            "fixture",
            "review-followups",
            "interpolation_start_suppressed.yml",
        ),
        // Lines 2, 3, 6, and 10.
        (
            "empty_grouping_sets",
            "fixture",
            "review-followups",
            "empty_grouping_sets.yml",
        ),
        // (not-in-subquery, 3) and (count-for-existence, 4).
        (
            "conflict_assignment",
            "fixture",
            "review-followups",
            "conflict_assignment.yml",
        ),
        // count-for-existence on lines 1, 2, 3, and 5.
        (
            "distinct_grouping_counts",
            "fixture",
            "review-followups",
            "distinct_grouping_counts.yml",
        ),
        // The finding stays on line 4, the EXISTS continuation, not an earlier template line.
        (
            "exists_continuation_line",
            "fixture",
            "review-followups",
            "exists_continuation_line.yml",
        ),
        // Scalar counts that can return no rows are not rewritten as EXISTS.
        (
            "count_may_return_no_rows_clean",
            "fixture",
            "review-followups",
            "count_may_return_no_rows_clean.yml",
        ),
        // CLI keeps literal-limit on lines 5, 7, 15, 20, 28, 30, and 34. The unit test still sees the raw lines 13, 23, and 37 before each directive suppresses only its own FETCH.
        (
            "offset_fetch_suppressed",
            "fixture",
            "review-followups",
            "offset_fetch_suppressed.yml",
        ),
        // CLI suppression is empty. The unit test still sees the raw literal-limit on line 5.
        (
            "table_fetch_suppressed",
            "fixture",
            "review-followups",
            "table_fetch_suppressed.yml",
        ),
    ]
    .map(|(label, kind, fixture, config)| {
        (
            label,
            rules_path(rule, kind, fixture),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_sql_shape_policy_bounded", &cases);
}
