use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "postgres-required-predicates";

#[test]
fn postgres_required_predicates_check_json_snapshot() {
    // Output cases formerly asserted via check_with_files in the rule unit module.

    let cases = vec![
        case("missing_predicate", "fail", "topics-001.yml"),
        // Lines 2 and 3 are distinct findings for the same standalone read.
        case("standalone_table_lines", "fail", "standalone-table.yml"),
        // Both table-set arms must remain findings on topics.
        case("table_set_arms", "fail", "table-arms.yml"),
        case("required_predicate_passes", "pass", "topics-001.yml"),
        case("left_join_passes", "pass", "left-join.yml"),
        case("left_join_missing", "fail", "left-join.yml"),
        case("embedded_query", "fail-embedded", "embedded-query.yml"),
        case("empty_relations", "fail", "empty-relations.yml"),
        // CLI suppression of sql/disabled.sql. The pre-suppression count stays in honors_disable_comments.
        case("disabled_sql", "fail", "disabled-sql.yml"),
        // exclude drops the only selected statement.
        case("exclude_selected_sql", "fail", "exclude-001.yml"),
        case("include_selected_sql", "fail", "include-001.yml"),
        case("dynamic_unrecoverable", "fail-dynamic", "dynamic-query.yml"),
        case("dynamic_ignored", "fail-dynamic", "dynamic-ignore.yml"),
        case("unparseable_sql", "fail", "unparseable.yml"),
        // The archived_by_id equality hint must mention `archived_by_id IS NULL` and requireColumns.
        case(
            "equality_is_null_hint",
            "fail-equality",
            "equality-archived.yml",
        ),
        // The unrelated query must not mention requireColumns.
        case(
            "equality_unrelated_no_hint",
            "fail-equality",
            "equality-unrelated.yml",
        ),
        // topics at line 9 and orders at line 17 must both remain.
        case("tagged_template", "fail-tagged", "tagged-template.yml"),
        // CLI suppression. The pre-suppression count of 2 stays in the unit test.
        case("tagged_suppressed", "fail-tagged", "tagged-suppressed.yml"),
        case(
            "imported_tag_unanalyzable",
            "fail-tagged",
            "imported-tag.yml",
        ),
        case(
            "trusted_named_tag",
            "fail-tagged",
            "trusted-imported-tag.yml",
        ),
        case(
            "untrusted_imported_tag",
            "fail-tagged",
            "base-imported-tag.yml",
        ),
        // Without a trusted tag the in-SQL directive does not suppress the unanalyzable finding.
        case(
            "untrusted_suppressed_tag",
            "fail-tagged",
            "base-suppressed-tag.yml",
        ),
        // CLI suppression once the tag is trusted. The pre-suppression anchor stays in the unit test.
        case(
            "trusted_suppressed_tag",
            "fail-tagged",
            "trusted-suppressed-tag.yml",
        ),
        case("other_module_tag", "fail-tagged", "other-module-tag.yml"),
        case("tag_subpath", "fail-tagged", "tag-subpath.yml"),
        case("tag_sibling", "fail-tagged", "tag-sibling.yml"),
        // Both document requirements must remain on line 9.
        case(
            "generic_argument_only",
            "fail-scoped-tagged",
            "generic-only.yml",
        ),
        // Both document requirements must remain on line 6.
        case(
            "leading_comment_only",
            "fail-scoped-tagged",
            "comment-only.yml",
        ),
        // Both document requirements must remain on line 6.
        case(
            "typed_parameter_executor",
            "fail-scoped-tagged",
            "typed-param-only.yml",
        ),
        // Both document requirements must remain on line 9.
        case(
            "all_triggers_combined",
            "fail-scoped-tagged",
            "combined.yml",
        ),
        // CLI suppression. The pre-suppression count of 2 stays in the unit test.
        case(
            "combined_suppressed",
            "fail-scoped-tagged",
            "combined-suppressed.yml",
        ),
        case("self_in_join", "deferred", "orders-self_in_join.yml"),
        case(
            "outer_right_join",
            "deferred",
            "orders-outer_right_join.yml",
        ),
        case("outer_full_join", "deferred", "orders-outer_full_join.yml"),
        case(
            "inner_left_join_passes",
            "deferred",
            "orders-inner_left_join_passes.yml",
        ),
        case(
            "self_unaliased_join_passes",
            "deferred",
            "orders-self_unaliased_join_passes.yml",
        ),
        // All four statement lines must remain findings.
        case(
            "query_expressions",
            "deferred",
            "orders-query_expressions.yml",
        ),
        case("cte_scope", "deferred", "orders-cte_scope.yml"),
        // Both UPDATE findings and both DELETE findings must remain.
        case("cte_mutations", "deferred", "orders-cte_mutations.yml"),
        // Both returning reads must remain findings.
        case("returning_reads", "deferred", "orders-returning_reads.yml"),
        case(
            "relation_locations",
            "deferred",
            "orders-relation_locations.yml",
        ),
        // public.events and archive.events must both remain.
        case(
            "qualified_partitions",
            "deferred",
            "qualified-partitions.yml",
        ),
        case("quoted_partition", "deferred", "quoted-partition.yml"),
        // The expression key is reported once, with account_id and missing physicalPartition.key.
        case("mixed_partition_keys", "deferred", "mixed-keys.yml"),
        // account_id must remain. The cannot-derive expression finding must be absent.
        case("mixed_allow", "deferred", "mixed-allow.yml"),
        // The stale partitionKeyExemptions entry and the stale allow entry must both remain.
        case("mixed_stale_allow", "deferred", "mixed-stale.yml"),
        // Lines 5, 7, and 9 must all remain on the host file.
        case("rebased_query_lines", "deferred", "rebased-queries.yml"),
        case("fail_kind", "columns", "columns-fail_kind.yml"),
        case("fail_join", "columns", "columns-fail_join.yml"),
        case("fail_or", "columns", "columns-fail_or.yml"),
        case("fail_delete", "columns", "columns-fail_delete.yml"),
        case("fail_self", "columns", "columns-fail_self.yml"),
        case("fail_insert", "columns", "columns-fail_insert.yml"),
        case("fail_null", "columns", "columns-fail_null.yml"),
        case("unknown_peer", "columns", "columns-unknown_peer.yml"),
        // Both the partition-key finding and the orders account_id finding must remain. repeated_scans_are_identical keeps the two-call identity.
        case(
            "ambiguous_columns",
            "columns",
            "columns-ambiguous_columns.yml",
        ),
        case("pass_eq", "columns", "columns-pass_eq.yml"),
        case("pass_any", "columns", "columns-pass_any.yml"),
        case("pass_join", "columns", "columns-pass_join.yml"),
        case("pass_or", "columns", "columns-pass_or.yml"),
        case("pass_update", "columns", "columns-pass_update.yml"),
        case("pass_in", "columns", "columns-pass_in.yml"),
        case("pass_cte", "columns", "columns-pass_cte.yml"),
        case("pass_qualified", "columns", "columns-pass_qualified.yml"),
        case(
            "pass_unqualified",
            "columns",
            "columns-pass_unqualified.yml",
        ),
        case("pass_between", "columns", "columns-pass_between.yml"),
        case("pass_bare", "columns", "columns-pass_bare.yml"),
        case("pass_unknown", "columns", "columns-pass_unknown.yml"),
        // CLI suppression. The pre-suppression non-empty count stays in suppression_directives_hide_sql_findings.
        case("suppress_next_line", "suppress", "suppress_next_line.yml"),
        // CLI suppression. The pre-suppression non-empty count stays in suppression_directives_hide_sql_findings.
        case("suppress_line", "suppress", "suppress_line.yml"),
        // CLI suppression. The pre-suppression non-empty count stays in suppression_directives_hide_sql_findings.
        case("suppress_file", "suppress", "suppress_file.yml"),
        case("textual_require", "fail", "textual-require.yml"),
        // Messages must not contain "partition key".
        case("partition_keys_off", "columns", "partition-keys-off.yml"),
        // The schema.json expression-key finding must remain.
        case(
            "expression_partition_key",
            "expression",
            "expression-key.yml",
        ),
        case("expression_exempt", "expression", "expression-exempt.yml"),
        case("stale_exemption", "stale-exemption", "stale-exemption.yml"),
        // Neither "cannot derive" nor a stale allow entry may appear.
        case("allow_reviewed", "allow", "allow-reviewed.yml"),
        // The stale allow entry and the cannot-derive finding must both remain.
        case("allow_stale", "allow", "allow-stale.yml"),
        // Post-suppression report: the caller finding and both helper imports must remain. The pre-suppression count of 5 stays in the unit test.
        case(
            "variant_merged_suppression",
            "variant-merged-suppression",
            "variant-merged.yml",
        ),
    ];
    assert_saved_fixture("postgres_required_predicates", &cases);
}

fn case<'a>(
    label: &'a str,
    fixture: &'a str,
    config: &'a str,
) -> (&'a str, std::path::PathBuf, std::path::PathBuf) {
    (
        label,
        rules_path(RULE, "fixture", fixture),
        rules_path(RULE, "configs", config),
    )
}
