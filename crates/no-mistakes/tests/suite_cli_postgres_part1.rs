#[path = "common/check_json_clean.rs"]
mod check_json_clean;
#[path = "common/check_json_snapshot.rs"]
mod check_json_snapshot;
#[path = "cli_mrf_snapshot_file_extension_policy.rs"]
mod cli_mrf_snapshot_file_extension_policy;
#[path = "cli_mrf_snapshot_markdown_child_links.rs"]
mod cli_mrf_snapshot_markdown_child_links;
#[path = "cli_mrf_snapshot_markdown_eval_tests.rs"]
mod cli_mrf_snapshot_markdown_eval_tests;
#[path = "cli_mrf_snapshot_markdown_link_display_text.rs"]
mod cli_mrf_snapshot_markdown_link_display_text;
#[cfg(feature = "mermaid-validation")]
#[path = "cli_mrf_snapshot_markdown_mermaid.rs"]
mod cli_mrf_snapshot_markdown_mermaid;
#[path = "cli_mrf_snapshot_markdown_reachability.rs"]
mod cli_mrf_snapshot_markdown_reachability;
#[path = "cli_mrf_snapshot_markdown_structure_budget.rs"]
mod cli_mrf_snapshot_markdown_structure_budget;
#[path = "cli_mrf_snapshot_rust_max_lines.rs"]
mod cli_mrf_snapshot_rust_max_lines;
#[path = "cli_mrf_snapshot_rust_no_inline_allows.rs"]
mod cli_mrf_snapshot_rust_no_inline_allows;
#[path = "cli_mrf_snapshot_rust_no_inline_tests.rs"]
mod cli_mrf_snapshot_rust_no_inline_tests;
#[path = "cli_postgres_detached_executors.rs"]
mod cli_postgres_detached_executors;
#[path = "cli_postgres_executor_obligations.rs"]
mod cli_postgres_executor_obligations;
#[path = "cli_postgres_non_call_references.rs"]
mod cli_postgres_non_call_references;
#[path = "cli_postgres_snapshot_appended_fragment.rs"]
mod cli_postgres_snapshot_appended_fragment;
#[path = "cli_postgres_snapshot_array_columns.rs"]
mod cli_postgres_snapshot_array_columns;
#[path = "cli_postgres_snapshot_bounded_chains.rs"]
mod cli_postgres_snapshot_bounded_chains;
#[path = "cli_postgres_snapshot_bounded_domain_rows.rs"]
mod cli_postgres_snapshot_bounded_domain_rows;
#[path = "cli_postgres_snapshot_bounded_false_positives.rs"]
mod cli_postgres_snapshot_bounded_false_positives;
#[path = "cli_postgres_snapshot_bounded_issue_3395.rs"]
mod cli_postgres_snapshot_bounded_issue_3395;
#[path = "cli_postgres_snapshot_bounded_issue_3432.rs"]
mod cli_postgres_snapshot_bounded_issue_3432;
#[path = "cli_postgres_snapshot_bounded_issue_3433.rs"]
mod cli_postgres_snapshot_bounded_issue_3433;
#[path = "cli_postgres_snapshot_bounded_issue_3436.rs"]
mod cli_postgres_snapshot_bounded_issue_3436;
#[path = "cli_postgres_snapshot_bounded_issue_3445.rs"]
mod cli_postgres_snapshot_bounded_issue_3445;
#[path = "cli_postgres_snapshot_bounded_issue_3459.rs"]
mod cli_postgres_snapshot_bounded_issue_3459;
#[path = "cli_postgres_snapshot_bounded_on.rs"]
mod cli_postgres_snapshot_bounded_on;
#[path = "cli_postgres_snapshot_bounded_statements.rs"]
mod cli_postgres_snapshot_bounded_statements;
#[path = "cli_postgres_snapshot_bounded_statements_more.rs"]
mod cli_postgres_snapshot_bounded_statements_more;
#[path = "cli_postgres_snapshot_callback_forwarded_literals.rs"]
mod cli_postgres_snapshot_callback_forwarded_literals;
#[path = "cli_postgres_snapshot_closed_candidates.rs"]
mod cli_postgres_snapshot_closed_candidates;
#[path = "cli_postgres_snapshot_closed_identifier_correlation.rs"]
mod cli_postgres_snapshot_closed_identifier_correlation;
#[path = "cli_postgres_snapshot_closed_identifiers.rs"]
mod cli_postgres_snapshot_closed_identifiers;
#[path = "cli_postgres_snapshot_closed_record_getter.rs"]
mod cli_postgres_snapshot_closed_record_getter;
#[path = "cli_postgres_snapshot_closed_record_prefix.rs"]
mod cli_postgres_snapshot_closed_record_prefix;
#[path = "cli_postgres_snapshot_column_naming.rs"]
mod cli_postgres_snapshot_column_naming;
#[path = "cli_postgres_snapshot_column_naming_more.rs"]
mod cli_postgres_snapshot_column_naming_more;
#[path = "cli_postgres_snapshot_column_requires_trigger.rs"]
mod cli_postgres_snapshot_column_requires_trigger;
#[path = "cli_postgres_snapshot_conflict_ordering.rs"]
mod cli_postgres_snapshot_conflict_ordering;
#[path = "cli_postgres_snapshot_constraint_validate.rs"]
mod cli_postgres_snapshot_constraint_validate;
#[path = "cli_postgres_snapshot_duplicate_function_body.rs"]
mod cli_postgres_snapshot_duplicate_function_body;
#[path = "cli_postgres_snapshot_explicit_columns.rs"]
mod cli_postgres_snapshot_explicit_columns;
#[path = "cli_postgres_snapshot_finite_text_columns.rs"]
mod cli_postgres_snapshot_finite_text_columns;
#[path = "cli_postgres_snapshot_fk_index.rs"]
mod cli_postgres_snapshot_fk_index;
#[path = "cli_postgres_snapshot_fk_on_delete.rs"]
mod cli_postgres_snapshot_fk_on_delete;
#[path = "cli_postgres_snapshot_frozen_record_dynamic.rs"]
mod cli_postgres_snapshot_frozen_record_dynamic;
#[path = "cli_postgres_snapshot_generated_column_predicates.rs"]
mod cli_postgres_snapshot_generated_column_predicates;
#[path = "cli_postgres_snapshot_generated_column_writes.rs"]
mod cli_postgres_snapshot_generated_column_writes;
#[path = "cli_postgres_snapshot_generated_column_writes_positional.rs"]
mod cli_postgres_snapshot_generated_column_writes_positional;
#[path = "cli_postgres_snapshot_generated_column_writes_shapes.rs"]
mod cli_postgres_snapshot_generated_column_writes_shapes;
#[path = "cli_postgres_snapshot_generated_identifier_returns.rs"]
mod cli_postgres_snapshot_generated_identifier_returns;
#[path = "cli_postgres_snapshot_generated_identifiers.rs"]
mod cli_postgres_snapshot_generated_identifiers;
#[path = "cli_postgres_snapshot_grouping_not_aggregation.rs"]
mod cli_postgres_snapshot_grouping_not_aggregation;
#[path = "cli_postgres_snapshot_idempotent_insert.rs"]
mod cli_postgres_snapshot_idempotent_insert;
#[path = "cli_postgres_snapshot_identifier_builders.rs"]
mod cli_postgres_snapshot_identifier_builders;
#[path = "cli_postgres_snapshot_identifier_catalogs.rs"]
mod cli_postgres_snapshot_identifier_catalogs;
#[path = "cli_postgres_snapshot_identifier_holes.rs"]
mod cli_postgres_snapshot_identifier_holes;
#[path = "cli_postgres_snapshot_identifier_length.rs"]
mod cli_postgres_snapshot_identifier_length;
#[path = "cli_postgres_snapshot_identifier_repros.rs"]
mod cli_postgres_snapshot_identifier_repros;
#[path = "cli_postgres_snapshot_issue_3030_schema.rs"]
mod cli_postgres_snapshot_issue_3030_schema;
#[path = "cli_postgres_snapshot_issue_3037.rs"]
mod cli_postgres_snapshot_issue_3037;
#[path = "cli_postgres_snapshot_issue_3046.rs"]
mod cli_postgres_snapshot_issue_3046;
#[path = "cli_postgres_snapshot_issue_3114.rs"]
mod cli_postgres_snapshot_issue_3114;
#[path = "cli_postgres_snapshot_issue_3114_exact.rs"]
mod cli_postgres_snapshot_issue_3114_exact;
#[path = "cli_postgres_snapshot_issue_3351.rs"]
mod cli_postgres_snapshot_issue_3351;
#[path = "cli_postgres_snapshot_issue_3394.rs"]
mod cli_postgres_snapshot_issue_3394;
#[path = "cli_postgres_snapshot_issue_3396.rs"]
mod cli_postgres_snapshot_issue_3396;
#[path = "cli_postgres_snapshot_issue_3397.rs"]
mod cli_postgres_snapshot_issue_3397;
#[path = "cli_postgres_snapshot_issue_3437.rs"]
mod cli_postgres_snapshot_issue_3437;
#[path = "cli_postgres_snapshot_issue_3452.rs"]
mod cli_postgres_snapshot_issue_3452;
#[path = "cli_postgres_snapshot_issue_3462.rs"]
mod cli_postgres_snapshot_issue_3462;
#[path = "cli_postgres_snapshot_issue_3476.rs"]
mod cli_postgres_snapshot_issue_3476;
#[path = "cli_postgres_snapshot_json_relation_ids.rs"]
mod cli_postgres_snapshot_json_relation_ids;
#[path = "cli_postgres_snapshot_key_column_types.rs"]
mod cli_postgres_snapshot_key_column_types;
#[path = "cli_postgres_snapshot_lock_ordering.rs"]
mod cli_postgres_snapshot_lock_ordering;
#[path = "cli_postgres_snapshot_lock_ordering_abstain.rs"]
mod cli_postgres_snapshot_lock_ordering_abstain;
#[path = "cli_postgres_snapshot_named_constraints.rs"]
mod cli_postgres_snapshot_named_constraints;
#[path = "cli_postgres_snapshot_no_add_column.rs"]
mod cli_postgres_snapshot_no_add_column;
#[path = "cli_postgres_snapshot_no_mutable_sql_registry.rs"]
mod cli_postgres_snapshot_no_mutable_sql_registry;
#[path = "cli_postgres_snapshot_no_offset.rs"]
mod cli_postgres_snapshot_no_offset;
#[path = "cli_postgres_snapshot_no_test_state_mutation.rs"]
mod cli_postgres_snapshot_no_test_state_mutation;
#[path = "cli_postgres_snapshot_no_unbounded_sql_identifier.rs"]
mod cli_postgres_snapshot_no_unbounded_sql_identifier;
#[path = "cli_postgres_snapshot_numeric_ternary.rs"]
mod cli_postgres_snapshot_numeric_ternary;
#[path = "cli_postgres_snapshot_object_naming.rs"]
mod cli_postgres_snapshot_object_naming;
#[path = "cli_postgres_snapshot_opaque_functions.rs"]
mod cli_postgres_snapshot_opaque_functions;
#[path = "cli_postgres_snapshot_redundant_index.rs"]
mod cli_postgres_snapshot_redundant_index;
#[path = "cli_postgres_snapshot_reopen_chains.rs"]
mod cli_postgres_snapshot_reopen_chains;
#[path = "cli_postgres_snapshot_reopen_shapes.rs"]
mod cli_postgres_snapshot_reopen_shapes;
#[path = "cli_postgres_snapshot_destructured_guard.rs"]
mod cli_postgres_snapshot_destructured_guard;
#[path = "cli_postgres_snapshot_soundness.rs"]
mod cli_postgres_snapshot_soundness;
#[path = "cli_postgres_snapshot_require_query_annotation.rs"]
mod cli_postgres_snapshot_require_query_annotation;
#[path = "cli_postgres_snapshot_required_comments.rs"]
mod cli_postgres_snapshot_required_comments;

#[path = "cli_check_rules_postgres_variants/correlated_member_guards.rs"]
mod correlated_member_guards;
