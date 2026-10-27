use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_no_generated_column_writes_check_json_snapshot() {
    let rule = "postgres-no-generated-column-writes";
    // Output formerly asserted by check / check_with_files. Suppression cases
    // are the CLI report after disable comments; pre-suppression counts stay
    // in the unit module.
    let rows = [
        // each_dml_shape also covers check_entry_point_uses_discovery.
        // fail-update.ts, fail-insert-cols.ts, fail-insert-columnless.ts,
        // fail-upsert.ts, and fail-merge.sql must all remain, each importing
        // items.created_at. pass.ts and schema.sql must stay absent.
        ("each_dml_shape", "fixture", ".", "fixture-default.yml"),
        (
            "extra_generated_columns_absent",
            "unit-fixture",
            "extra",
            "extra-votes.yml",
        ),
        // The stale extraGeneratedColumns message must remain.
        (
            "stale_extra_generated_columns",
            "fixture",
            ".",
            "fixture-stale-extra.yml",
        ),
        (
            "chr_encoded_updates",
            "unit-fixture",
            "chr-encoded",
            "schema-ts.yml",
        ),
        // Every finding is fail-update.ts, and the set is non-empty.
        (
            "only_fail_update",
            "fixture",
            ".",
            "fixture-only-update.yml",
        ),
        (
            "custom_executor_bindings",
            "unit-fixture",
            "custom-executor",
            "custom-executor.yml",
        ),
        (
            "missing_file_skipped",
            "fixture",
            ".",
            "fixture-missing.yml",
        ),
        (
            "schema_without_dml",
            "fixture",
            ".",
            "fixture-schema-only.yml",
        ),
        (
            "plain_schema",
            "unit-fixture",
            "plain-schema",
            "plain-schema.yml",
        ),
        (
            "dynamic_empty_executors",
            "unit-fixture",
            "dynamic",
            "dynamic-empty-executors.yml",
        ),
        // Definite writes stay empty. Schema-marker findings must remain.
        (
            "invalid_schema_markers",
            "unit-fixture",
            "invalid-schema",
            "invalid-schema.yml",
        ),
        (
            "mixed_unparseable",
            "unit-fixture",
            "mixed-unparseable",
            "mixed-unparseable.yml",
        ),
        ("merge_sql_message", "fixture", ".", "fixture-merge-sql.yml"),
        // Five findings on lines 1, 2, 3, 3, 4: two trigger-maintained and three generated.
        (
            "if_not_exists",
            "unit-fixture",
            "if-not-exists",
            "schema-writes-trigger.yml",
        ),
        (
            "qualified_lifecycles",
            "unit-fixture",
            "qualified-lifecycle",
            "schema-writes.yml",
        ),
        (
            "unqualified_alter_and_drop",
            "unit-fixture",
            "unqualified-lifecycle",
            "schema-writes.yml",
        ),
        (
            "abort_chain",
            "unit-fixture",
            "abort-chain",
            "schema-writes.yml",
        ),
        (
            "migration_numeric_order",
            "unit-fixture",
            "migration-order",
            "migration-order.yml",
        ),
        (
            "dormant_conditional_ddl",
            "unit-fixture",
            "table-event-forms",
            "execution-sql.yml",
        ),
        (
            "dormant_only_events",
            "unit-fixture",
            "table-event-forms",
            "dormant-only.yml",
        ),
        (
            "conditional_alter",
            "unit-fixture",
            "conditional-alter",
            "schema-writes.yml",
        ),
        (
            "global_and_local_temp",
            "unit-fixture",
            "global-local",
            "schema-writes.yml",
        ),
        // Messages must name trigger-maintained orders.updated_at and each fail file.
        (
            "trigger_invalid_writes",
            "unit-fixture",
            "trigger-maintained",
            "trigger-invalid-writes.yml",
        ),
        (
            "trigger_reads_quiet",
            "unit-fixture",
            "trigger-maintained",
            "trigger-reads.yml",
        ),
        (
            "trigger_empty_list",
            "unit-fixture",
            "trigger-maintained",
            "trigger-empty-list.yml",
        ),
        (
            "trigger_upper_sql",
            "unit-fixture",
            "trigger-maintained",
            "trigger-upper-sql.yml",
        ),
        (
            "trigger_upper_config",
            "unit-fixture",
            "trigger-maintained",
            "trigger-upper-config.yml",
        ),
        // Must contain generated `items.created_at` and must not contain
        // trigger-maintained `items.created_at`.
        (
            "trigger_generated_message",
            "unit-fixture",
            "trigger-maintained",
            "trigger-generated.yml",
        ),
        (
            "trigger_twice_one_finding",
            "unit-fixture",
            "trigger-maintained",
            "trigger-twice.yml",
        ),
        // Both updated_at and modified_at must remain.
        (
            "trigger_each_listed_column",
            "unit-fixture",
            "trigger-maintained",
            "trigger-both.yml",
        ),
        (
            "trigger_stale_entry",
            "unit-fixture",
            "trigger-maintained",
            "trigger-stale.yml",
        ),
        (
            "trigger_suppressions",
            "unit-fixture",
            "trigger-maintained",
            "trigger-suppress.yml",
        ),
        // writes.sql lines 1–7 and 9 must remain: the first three and the last
        // three are trigger-maintained, and lines 4 and 5 are generated.
        (
            "review_regression_writes",
            "unit-fixture",
            "review-regressions",
            "schema-writes-trigger.yml",
        ),
        // Two target updated_at findings on lines 1 and 2 must remain, both
        // trigger-maintained orders.updated_at.
        (
            "alter_columns_not_stale",
            "unit-fixture",
            "alter-trigger",
            "schema-writes-trigger.yml",
        ),
        (
            "recreated_generated",
            "unit-fixture",
            "recreated-generated",
            "schema-writes-trigger.yml",
        ),
        (
            "alter_added_generated",
            "unit-fixture",
            "alter-generated-order",
            "schema-writes.yml",
        ),
        (
            "recreated_trigger",
            "unit-fixture",
            "recreated-trigger",
            "schema-writes-trigger.yml",
        ),
        (
            "dynamic_recreation",
            "unit-fixture",
            "dynamic-recreation",
            "schema-writes-trigger.yml",
        ),
        // Post-suppression definite writes stay on lines 2, 6, 7, and 8.
        // Line 4 is suppressed. The line 5 schema marker must remain.
        (
            "dollar_identifiers_suppressed",
            "unit-fixture",
            "dollar-identifiers",
            "schema-writes-trigger.yml",
        ),
        // Post-suppression lines 3 and 8 remain. Lines 6 and 7 are suppressed.
        (
            "write_occurrences_suppressed",
            "unit-fixture",
            "write-occurrences",
            "schema-writes-trigger.yml",
        ),
        // writes.sql and write.ts each keep one generated and one trigger-maintained finding.
        (
            "both_column_kinds",
            "unit-fixture",
            "both-column-kinds",
            "both-kinds.yml",
        ),
        (
            "uppercase_generated",
            "unit-fixture",
            "both-column-kinds",
            "both-upper.yml",
        ),
        (
            "alter_external",
            "unit-fixture",
            "alter-trigger",
            "alter-external.yml",
        ),
        (
            "quoted_identity_lines",
            "unit-fixture",
            "quoted-identity",
            "schema-writes.yml",
        ),
        // Imports "ORDERS".computed and "Orders".computed must both remain.
        (
            "quoted_embedded_identities",
            "unit-fixture",
            "quoted-identity",
            "quoted-embedded.yml",
        ),
        (
            "temporary_drop",
            "unit-fixture",
            "temporary-drop",
            "schema-writes.yml",
        ),
        (
            "temporary_shadow",
            "unit-fixture",
            "temporary-shadow",
            "schema-writes.yml",
        ),
        (
            "exception_handlers",
            "unit-fixture",
            "exception-handlers",
            "schema-writes.yml",
        ),
        (
            "transaction_rollback",
            "unit-fixture",
            "transaction-rollback",
            "schema-writes.yml",
        ),
        (
            "format_identities",
            "unit-fixture",
            "format-identities",
            "schema-writes.yml",
        ),
        (
            "conditional_assignment",
            "unit-fixture",
            "conditional-assignment",
            "schema-writes.yml",
        ),
        // Lines 1, 2, 3, 4, 7, 8, 9, 10, and 11 must remain, and no message is stale.
        (
            "configured_identities",
            "unit-fixture",
            "configured-identities",
            "configured-identities.yml",
        ),
        (
            "ctas_order",
            "unit-fixture",
            "ctas-order",
            "schema-writes.yml",
        ),
        // Target total must remain. A valid RAISE is not a schema marker.
        (
            "raise_exception",
            "unit-fixture",
            "raise-exception",
            "schema-writes.yml",
        ),
        (
            "quoted_column_alter",
            "unit-fixture",
            "quoted-column-alter",
            "schema-writes.yml",
        ),
        // Messages must not contain "could not be analyzed".
        (
            "nested_shadow",
            "unit-fixture",
            "nested-shadow",
            "schema-writes.yml",
        ),
        (
            "create_if_not_exists",
            "unit-fixture",
            "create-if-not-exists",
            "schema-writes.yml",
        ),
        // The unconditional recreate replaces the generated table, so no finding remains.
        (
            "quoted_control_keyword",
            "unit-fixture",
            "quoted-control-keyword",
            "schema-writes.yml",
        ),
        (
            "migration_write_history",
            "unit-fixture",
            "migration-write-history",
            "migration-history.yml",
        ),
        (
            "same_line_ddl",
            "unit-fixture",
            "same-line-migration",
            "same-line.yml",
        ),
        (
            "labeled_shadow",
            "unit-fixture",
            "labeled-shadow",
            "schema-writes.yml",
        ),
        // The temporary shadow makes the write ordinary.
        (
            "temp_if_not_exists",
            "unit-fixture",
            "temp-if-not-exists",
            "schema-writes.yml",
        ),
        (
            // CLI rejects importSpecifier: null. The rule treats null as the
            // empty specifier, so "" plus executorNames: [] is the same
            // no-profile report (explicit_empty_names_select_no_executor_profile).
            "empty_executor_names",
            "unit-fixture",
            "quoted-identity",
            "empty-executor-names.yml",
        ),
        // Post-suppression lines 7 and 8 remain. Lines 5 and 6 are suppressed.
        (
            "do_writes_suppressed",
            "unit-fixture",
            "do-write-lines",
            "do-writes.yml",
        ),
        // Lines 5, 10, 15, 20, 39, and 42 stay, target unanalyzable.
        // The first message starts `write.ts:5:` and contains `unanalyzableSql: ignore`.
        // Dynamic SELECT/DELETE and the suppressed call stay quiet.
        (
            "unanalyzable_default",
            "unit-fixture",
            "unanalyzable",
            "schema-ts.yml",
        ),
        (
            "unanalyzable_ignore",
            "unit-fixture",
            "unanalyzable",
            "unanalyzable-ignore.yml",
        ),
        (
            "unanalyzable_write_only",
            "unit-fixture",
            "unanalyzable",
            "unanalyzable-write-only.yml",
        ),
        (
            "unanalyzable_ordinary",
            "unit-fixture",
            "unanalyzable",
            "unanalyzable-ordinary.yml",
        ),
    ];
    let cases = rows.map(|(label, kind, name, config)| {
        (
            label,
            rules_path(rule, kind, name),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_no_generated_column_writes", &cases);
}
