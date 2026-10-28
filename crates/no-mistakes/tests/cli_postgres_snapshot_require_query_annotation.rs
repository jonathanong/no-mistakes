use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "postgres-require-query-annotation";

#[test]
#[rustfmt::skip]
fn postgres_require_query_annotation_check_json_snapshot() {
    // Output cases formerly asserted via check_with_files in the rule unit module.
    let cases = vec![
        case("missing_annotation", "fail", "example-db-query-ts.yml"),
        case("annotated_query", "pass-annotated", "example-db-query-ts.yml"),
        case("transaction_commands", "pass-transaction", "example-db-query-ts.yml"),
        case("line_comment_is_not_an_annotation", "fail-line-comment", "example-db-query-ts.yml"),
        // CLI suppression of src/disabled.ts. The pre-suppression count stays in honors_disable_comments.
        case("disabled_ts", "fail", "example-db-disabled-ts.yml"),
        // // finding: missing lines must say query execution must start; other // finding: lines must say leading SQL is unanalyzable.
        case("chained_executor_return", "chained-executor-return", "chained-write.yml"),
        // Opaque bindings stay ignored. declaration_line and known/unknown SQL text stay in the unit test.
        case("var_scopes", "var-scopes", "var-scopes.yml"),
        // exclude drops the only selected file.
        case("exclude_query", "fail", "exclude-query.yml"),
        case("include_misses_file", "fail", "include-missing.yml"),
        case("custom_executor_misses", "fail", "custom-executor.yml"),
        // Lines 3, 5, 8, 10, and 12 must not be findings. Lines 6 and 13 must remain must-start findings. Report also keeps four leading-unanalyzable findings; ignore must not.
        case("nested_fragments_report", "nested-fragments", "nested-report.yml"),
        case("nested_fragments_ignore", "nested-fragments", "nested-ignore.yml"),
        // Report includes // unknown lines as leading-unanalyzable. Ignore must not.
        case("scalar_fragments_report", "scalar-fragment-values", "scalar-report.yml"),
        case("scalar_fragments_ignore", "scalar-fragment-values", "scalar-ignore.yml"),
        // Lines follow // finding:, plus // unanalyzable: only when the policy is report. Both joined-binding fixtures keep the missing-head finding under either policy.
        case("post_merge_report", "helper-tracing-post-merge", "app-db-query-mts-report.yml"),
        case("post_merge_ignore", "helper-tracing-post-merge", "app-db-query-mts-ignore.yml"),
        case("tag_callback_report", "helper-tracing-tag-callback", "app-db-query-mts-report.yml"),
        case("tag_callback_ignore", "helper-tracing-tag-callback", "app-db-query-mts-ignore.yml"),
        case("alternative_callback_report", "helper-tracing-alternative-callback", "app-db-query-mts-report.yml"),
        case("alternative_callback_ignore", "helper-tracing-alternative-callback", "app-db-query-mts-ignore.yml"),
        case("named_delete_report", "helper-tracing-named-delete", "app-db-query-mts-report.yml"),
        case("named_delete_ignore", "helper-tracing-named-delete", "app-db-query-mts-ignore.yml"),
        case("argument_members_report", "helper-tracing-argument-members", "app-db-query-mts-report.yml"),
        case("argument_members_ignore", "helper-tracing-argument-members", "app-db-query-mts-ignore.yml"),
        case("discarded_values_report", "helper-tracing-discarded-values", "app-db-query-mts-report.yml"),
        case("discarded_values_ignore", "helper-tracing-discarded-values", "app-db-query-mts-ignore.yml"),
        case("call_argument_order_report", "helper-tracing-call-argument-order", "app-db-query-mts-report.yml"),
        case("call_argument_order_ignore", "helper-tracing-call-argument-order", "app-db-query-mts-ignore.yml"),
        case("live_binding_report", "helper-tracing-live-binding", "app-db-query-mts-report.yml"),
        case("live_binding_ignore", "helper-tracing-live-binding", "app-db-query-mts-ignore.yml"),
        case("argument_slot_write_report", "helper-tracing-argument-slot-write", "app-db-query-mts-report.yml"),
        case("argument_slot_write_ignore", "helper-tracing-argument-slot-write", "app-db-query-mts-ignore.yml"),
        case("destructuring_alias_report", "helper-tracing-destructuring-alias", "app-db-query-mts-report.yml"),
        case("destructuring_alias_ignore", "helper-tracing-destructuring-alias", "app-db-query-mts-ignore.yml"),
        case("argument_length_report", "helper-tracing-argument-length", "app-db-query-mts-report.yml"),
        case("argument_length_ignore", "helper-tracing-argument-length", "app-db-query-mts-ignore.yml"),
        case("returned_callbacks_report", "helper-tracing-returned-callbacks", "app-db-query-mts-report.yml"),
        case("returned_callbacks_ignore", "helper-tracing-returned-callbacks", "app-db-query-mts-ignore.yml"),
        case("deleted_slot_callback_report", "helper-tracing-deleted-slot-callback", "app-db-query-mts-report.yml"),
        case("deleted_slot_callback_ignore", "helper-tracing-deleted-slot-callback", "app-db-query-mts-ignore.yml"),
        case("module_sibling_report", "helper-tracing-module-sibling", "app-db-query-mts-report.yml"),
        case("module_sibling_ignore", "helper-tracing-module-sibling", "app-db-query-mts-ignore.yml"),
        case("module_sibling_reverse_report", "helper-tracing-module-sibling-reverse", "app-db-query-mts-report.yml"),
        case("module_sibling_reverse_ignore", "helper-tracing-module-sibling-reverse", "app-db-query-mts-ignore.yml"),
        case("callback_installers_report", "helper-tracing-callback-installers", "app-db-query-mts-report.yml"),
        case("callback_installers_ignore", "helper-tracing-callback-installers", "app-db-query-mts-ignore.yml"),
        case("unary_argument_index_report", "helper-tracing-unary-argument-index", "app-db-query-mts-report.yml"),
        case("unary_argument_index_ignore", "helper-tracing-unary-argument-index", "app-db-query-mts-ignore.yml"),
        case("shared_snapshots_report", "helper-tracing-shared-snapshots", "app-db-query-mts-report.yml"),
        case("shared_snapshots_ignore", "helper-tracing-shared-snapshots", "app-db-query-mts-ignore.yml"),
        case("missing_alternative_binding_report", "helper-tracing-missing-alternative-binding", "app-db-query-mts-report.yml"),
        case("missing_alternative_binding_ignore", "helper-tracing-missing-alternative-binding", "app-db-query-mts-ignore.yml"),
        case("joined_binding_valid_report", "helper-tracing-joined-binding-valid", "app-db-query-mts-report.yml"),
        case("joined_binding_valid_ignore", "helper-tracing-joined-binding-valid", "app-db-query-mts-ignore.yml"),
        case("joined_binding_report", "helper-tracing-joined-binding", "app-db-query-mts-report.yml"),
        case("joined_binding_ignore", "helper-tracing-joined-binding", "app-db-query-mts-ignore.yml"),
        case("mapped_scalar_report", "helper-tracing-mapped-scalar", "app-db-query-cjs-report.yml"),
        case("mapped_scalar_ignore", "helper-tracing-mapped-scalar", "app-db-query-cjs-ignore.yml"),
        case("callback_revisit_report", "helper-tracing-callback-revisit", "app-db-query-cjs-report.yml"),
        case("callback_revisit_ignore", "helper-tracing-callback-revisit", "app-db-query-cjs-ignore.yml"),
        case("arm_module_report", "helper-tracing-arm-module", "app-db-arm-report.yml"),
        case("arm_module_ignore", "helper-tracing-arm-module", "app-db-arm-ignore.yml"),
        case("imported_callback_revisit_report", "helper-tracing-imported-callback-revisit", "app-db-imported-report.yml"),
        case("imported_callback_revisit_ignore", "helper-tracing-imported-callback-revisit", "app-db-imported-ignore.yml"),
        case("mapped_formal_callback_report", "helper-tracing-mapped-formal-callback", "app-db-formal-report.yml"),
        case("mapped_formal_callback_ignore", "helper-tracing-mapped-formal-callback", "app-db-formal-ignore.yml"),
        case("argument_alias_write_report", "helper-tracing-argument-alias-write", "app-db-sloppy-report.yml"),
        case("argument_alias_write_ignore", "helper-tracing-argument-alias-write", "app-db-sloppy-ignore.yml"),
        case("dynamic_delete_slot_write_report", "helper-tracing-dynamic-delete-slot-write", "app-db-sloppy-report.yml"),
        case("dynamic_delete_slot_write_ignore", "helper-tracing-dynamic-delete-slot-write", "app-db-sloppy-ignore.yml"),
        // --jobs 1 and 2 cover the former thread-pool identity. Report also keeps // unanalyzable: lines; ignore must not.
        case("helper_review_report", "helper-tracing-review", "helper-review-report.yml"),
        case("helper_review_ignore", "helper-tracing-review", "helper-review-ignore.yml"),
        case("cyclic_reexport_executor", "helper-tracing-reachability", "reachability-ignore.yml"),
        // Both indexed findings, lines 8 and 9, must remain.
        case("shared_container_snapshots", "helper-tracing-values-cow", "app-db-query-mts-report.yml"),
        case("helper_tracing_orders_report", "helper-tracing", "orders-report.yml"),
        case("helper_tracing_orders_ignore", "helper-tracing", "orders-ignore.yml"),
        case("helper_tracing_safety", "helper-tracing-safety", "safety-tags.yml"),
        case("helper_projection_parse", "helper-tracing-ownership", "projection-db.yml"),
        case("straight_line_mutations", "helper-tracing-coverage", "coverage-fragment.yml"),
        // Both policies report only packages/bad/src/query.mts line 3.
        case("monorepo_aliases_ignore", "helper-tracing-monorepo", "monorepo-ignore.yml"),
        case("monorepo_aliases_report", "helper-tracing-monorepo", "monorepo-report.yml"),
        // Max lines must equal // missing: markers. Low must include those lines and may report more. Observer counts and verdict statuses stay in the unit test.
        case("loop_query_low_ignore", "loop-carried-reassignment", "loop-query-low-ignore.yml"),
        case("loop_while_only_low_ignore", "loop-carried-reassignment", "loop-while-only-low-ignore.yml"),
        case("loop_query_low_advise", "loop-carried-reassignment", "loop-query-low-advise.yml"),
        case("loop_while_only_low_advise", "loop-carried-reassignment", "loop-while-only-low-advise.yml"),
        case("loop_query_low_report", "loop-carried-reassignment", "loop-query-low-report.yml"),
        case("loop_while_only_low_report", "loop-carried-reassignment", "loop-while-only-low-report.yml"),
        case("loop_query_max_ignore", "loop-carried-reassignment", "loop-query-max-ignore.yml"),
        case("loop_while_only_max_ignore", "loop-carried-reassignment", "loop-while-only-max-ignore.yml"),
        case("loop_query_max_advise", "loop-carried-reassignment", "loop-query-max-advise.yml"),
        case("loop_while_only_max_advise", "loop-carried-reassignment", "loop-while-only-max-advise.yml"),
        case("loop_query_max_report", "loop-carried-reassignment", "loop-query-max-report.yml"),
        case("loop_while_only_max_report", "loop-carried-reassignment", "loop-while-only-max-report.yml"),
        // Observer counters and the malformed parse stay in the unit test.
        case("loop_format_false_positive", "loop-carried-reassignment", "loop-format.yml"),
        // Low keeps live read(q) lines except low-unknown. Max keeps // missing: lines. Observer root counts stay in the unit test.
        case("loop_path_low", "loop-path-witness", "loop-path-low.yml"),
        case("loop_path_max", "loop-path-witness", "loop-path-max.yml"),
        // Effective effort equality stays in the unit test.
        case("loop_condition_low", "loop-condition-writes", "loop-condition-low.yml"),
        case("loop_condition_max", "loop-condition-writes", "loop-condition-max.yml"),
        // CLI suppression of both files. The pre-suppression count of 3 stays in the unit test.
        case("loop_carried_suppressed", "loop-carried-reassignment", "loop-suppressed.yml"),
        case("loop_tag_untrusted", "loop-carried-reassignment", "loop-tag-untrusted.yml"),
        case("loop_tag_configured_untrusted", "loop-carried-reassignment", "loop-tag-configured.yml"),
        case("loop_tag_trusted", "loop-carried-reassignment", "loop-tag-trusted.yml"),
        // Rule findings stay empty. SqlVerdictStatus stays in the unit test.
        case("provenance_advice", "provenance-advice", "provenance-advise.yml"),
        // The single rule finding must remain. The abstain verdict stays in the unit test.
        case("provenance_advice_helper", "provenance-advice-helper", "provenance-advise-helper.yml"),
        // rules line 7 must remain. sqlVerdicts are absent from check JSON and stay in the unit test.
        case("untrusted_tag_append", "verdict-invalid-append", "verdict-invalid-append.yml"),
        // Named trusted roots remain clear despite opaque suffixes; custom tags and escaped builders advise.
        case("append_opaque_prefix", "append-opaque-prefix", "append-opaque-prefix.yml"),
        // One fixed helper clears; a fixed missing head fails under both policies.
        case("helper_heads_report", "helper-heads", "helper-heads-report.yml"),
        case("helper_heads_ignore", "helper-heads", "helper-heads-ignore.yml"),
        // Direct source imports use prepared exported heads; unknown links advise or report.
        case("imported_helper_heads_report", "imported-helper-heads", "imported-helper-heads-report.yml"),
        case("imported_helper_heads_ignore", "imported-helper-heads", "imported-helper-heads-ignore.yml"),
        // Concise arrows and one-hop export aliases prove heads; consumed custom appends abstain.
        case("helper_arrow_heads_report", "helper-arrow-heads", "helper-arrow-heads-report.yml"),
        case("helper_arrow_heads_ignore", "helper-arrow-heads", "helper-arrow-heads-ignore.yml"),
        // Relative factory/type calls join only after resolver confirmation.
        case("relative_helper_heads_report", "relative-helper-heads", "relative-helper-heads-report.yml"),
        case("relative_helper_heads_ignore", "relative-helper-heads", "relative-helper-heads-ignore.yml"),
        // Finite local returns and prepared append effects keep their low heads.
        case("conditional_return_local_report", "low-heads", "conditional-return-local-report.yml"),
        case("conditional_return_local_ignore", "low-heads", "conditional-return-local-ignore.yml"),
        case("append_effects_report", "append-effects", "append-effects-report.yml"),
        case("append_effects_ignore", "append-effects", "append-effects-ignore.yml"),
        case("append_effect_composition_report", "low-heads", "append-effect-composition-report.yml"),
        case("append_effect_composition_ignore", "low-heads", "append-effect-composition-ignore.yml"),
        case("effect_defaults_report", "effect-defaults", "effect-defaults-report.yml"),
        // Captured callbacks may add suffixes even if replayed; escaping the builder still advises.
        case("captured_append_effects_report", "low-heads", "captured-append-effects-report.yml"),
        case("captured_prefix_report", "captured-prefix", "captured-prefix-report.yml"),
        case("captured_prefix_ignore", "captured-prefix", "captured-prefix-ignore.yml"),
        // Field and imported value projections retain named, missing, and advisory cases.
        case("text_field_report", "low-heads", "text-field-report.yml"),
        case("text_field_third_report", "low-heads", "text-field-third-report.yml"),
        case("captured_text_field_report", "low-heads", "captured-text-field-report.yml"),
        case("imported_string_heads_report", "imported-string-heads", "imported-string-heads-report.yml"),
        case("helper_records_report", "low-heads", "helper-records-report.yml"),
        case("record_text_report", "low-heads", "record-text-report.yml"),
        case("helper_records_ignore", "low-heads", "helper-records-ignore.yml"),
        case("imported_helper_records_report", "imported-helper-records", "imported-helper-records-report.yml"),
        case("imported_helper_records_ignore", "imported-helper-records", "imported-helper-records-ignore.yml"),
        case("returned_params_report", "returned-params", "returned-params-report.yml"),
        case("helper_bindings_report", "low-heads", "helper-bindings-report.yml"),
        case("helper_bindings_ignore", "low-heads", "helper-bindings-ignore.yml"),
        case("imported_helper_bindings_report", "imported-helper-bindings", "imported-helper-bindings-report.yml"),
        case("imported_helper_bindings_ignore", "imported-helper-bindings", "imported-helper-bindings-ignore.yml"),
        // Every return path must agree; only fresh trusted builders support append projection.
        case("finite_helper_returns_local", "low-heads", "finite-helper-returns-local-report.yml"),
        case("finite_helper_returns_imported_report", "finite-helper-returns", "finite-helper-returns-report.yml"),
        case("finite_helper_returns_imported_ignore", "finite-helper-returns", "finite-helper-returns-ignore.yml"),
        case("finite_helper_guards_report", "low-heads", "finite-helper-guards-report.yml"),
        case("finite_helper_guards_ignore", "low-heads", "finite-helper-guards-ignore.yml"),
        case("callback_heads_report", "callback-heads", "callback-heads-low-report.yml"),
        case("callback_heads_ignore", "callback-heads", "callback-heads-low-ignore.yml"),
        case("namespace_helper_heads_report", "namespace-helper-heads", "namespace-helper-heads-report.yml"),
        case("namespace_helper_heads_ignore", "namespace-helper-heads", "namespace-helper-heads-ignore.yml"),
        case("returned_captured_prefix_report", "low-heads", "returned-captured-prefix-report.yml"),
        case("returned_captured_prefix_ignore", "low-heads", "returned-captured-prefix-ignore.yml"),
        case("finite_comment_names_report", "low-heads", "finite-comment-names-report.yml"),
        case("finite_comment_names_ignore", "low-heads", "finite-comment-names-ignore.yml"),
        case("two_arm_assignment_report", "low-heads", "two-arm-assignment-report.yml"),
        case("callback_export_payload_report", "callback-export-payload", "callback-export-payload-report.yml"),
        case("callback_more_report", "callback-more", "callback-more-report.yml"),
    ];
    assert_saved_fixture("postgres_require_query_annotation", &cases);
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

#[test]
fn postgres_annotation_timings_check_json_snapshot() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-timings");
    assert_saved_fixture(
        "postgres_annotation_timings",
        &[("max_effort", root.clone(), root.join(".no-mistakes.yml"))],
    );
}
