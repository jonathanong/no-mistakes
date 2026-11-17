#[path = "cli_postgres_snapshot_required_predicates.rs"]
mod cli_postgres_snapshot_required_predicates;
#[path = "cli_postgres_snapshot_sql_shape_policy.rs"]
mod cli_postgres_snapshot_sql_shape_policy;
#[path = "cli_postgres_snapshot_sql_shape_policy_bounded.rs"]
mod cli_postgres_snapshot_sql_shape_policy_bounded;
#[path = "cli_postgres_snapshot_sql_shape_policy_returns.rs"]
mod cli_postgres_snapshot_sql_shape_policy_returns;
#[path = "cli_postgres_snapshot_sql_statement_policy.rs"]
mod cli_postgres_snapshot_sql_statement_policy;
#[path = "cli_postgres_snapshot_status_lifecycle.rs"]
mod cli_postgres_snapshot_status_lifecycle;
#[path = "cli_postgres_snapshot_table_shape.rs"]
mod cli_postgres_snapshot_table_shape;
#[path = "cli_postgres_snapshot_transformed_uuid_predicates.rs"]
mod cli_postgres_snapshot_transformed_uuid_predicates;
#[path = "cli_postgres_snapshot_unanalyzable_schema.rs"]
mod cli_postgres_snapshot_unanalyzable_schema;
#[path = "cli_postgres_snapshot_undeclared_executors.rs"]
mod cli_postgres_snapshot_undeclared_executors;
#[path = "cli_postgres_snapshot_unmatched_executors.rs"]
mod cli_postgres_snapshot_unmatched_executors;
#[path = "cli_postgres_snapshot_value_const_holes.rs"]
mod cli_postgres_snapshot_value_const_holes;

#[path = "cli_check_rules_postgres_json_relation_ids.rs"]
mod cli_check_rules_postgres_json_relation_ids;

#[path = "cli_check_postgres_statement_lines.rs"]
mod cli_check_postgres_statement_lines;

#[path = "cli_check_postgres_body_recovery.rs"]
mod cli_check_postgres_body_recovery;

#[path = "cli_check_postgres_release_blocker.rs"]
mod cli_check_postgres_release_blocker;

#[path = "cli_check_rules_postgres.rs"]
mod cli_check_rules_postgres;
#[path = "cli_check_rules_postgres_add_column.rs"]
mod cli_check_rules_postgres_add_column;
#[path = "cli_check_rules_postgres_annotation.rs"]
mod cli_check_rules_postgres_annotation;
#[path = "cli_check_rules_postgres_array.rs"]
mod cli_check_rules_postgres_array;
#[path = "cli_check_rules_postgres_bounded.rs"]
mod cli_check_rules_postgres_bounded;
#[path = "cli_check_rules_postgres_column_naming.rs"]
mod cli_check_rules_postgres_column_naming;
#[path = "cli_check_rules_postgres_column_trigger.rs"]
mod cli_check_rules_postgres_column_trigger;
#[path = "cli_check_rules_postgres_conflict.rs"]
mod cli_check_rules_postgres_conflict;
#[path = "cli_check_rules_postgres_ddl_unanalyzable.rs"]
mod cli_check_rules_postgres_ddl_unanalyzable;
#[path = "cli_check_rules_postgres_dml.rs"]
mod cli_check_rules_postgres_dml;
#[path = "cli_check_rules_postgres_duplicate_function.rs"]
mod cli_check_rules_postgres_duplicate_function;
#[path = "cli_check_rules_postgres_finite_text.rs"]
mod cli_check_rules_postgres_finite_text;
#[path = "cli_check_rules_postgres_fk_on_delete.rs"]
mod cli_check_rules_postgres_fk_on_delete;
#[path = "cli_check_rules_postgres_generated.rs"]
mod cli_check_rules_postgres_generated;
#[path = "cli_check_rules_postgres_helper_flow.rs"]
mod cli_check_rules_postgres_helper_flow;
#[path = "cli_check_rules_postgres_identifier_length.rs"]
mod cli_check_rules_postgres_identifier_length;
#[path = "cli_check_rules_postgres_imported_helpers.rs"]
mod cli_check_rules_postgres_imported_helpers;
#[path = "cli_check_rules_postgres_key_column_types.rs"]
mod cli_check_rules_postgres_key_column_types;
#[path = "cli_check_rules_postgres_migrations.rs"]
mod cli_check_rules_postgres_migrations;
#[path = "cli_check_rules_postgres_named_constraints.rs"]
mod cli_check_rules_postgres_named_constraints;
#[path = "cli_check_rules_postgres_object_naming.rs"]
mod cli_check_rules_postgres_object_naming;
#[path = "cli_check_rules_postgres_offset.rs"]
mod cli_check_rules_postgres_offset;
#[path = "cli_check_rules_postgres_regress_sql.rs"]
mod cli_check_rules_postgres_regress_sql;
#[path = "cli_check_rules_postgres_required_comments.rs"]
mod cli_check_rules_postgres_required_comments;
#[path = "cli_check_rules_postgres_sql_cross_file.rs"]
mod cli_check_rules_postgres_sql_cross_file;
#[path = "cli_check_rules_postgres_sql_shape_functions.rs"]
mod cli_check_rules_postgres_sql_shape_functions;
#[path = "cli_check_rules_postgres_sql_statement_policy.rs"]
mod cli_check_rules_postgres_sql_statement_policy;
#[path = "cli_check_rules_postgres_status_lifecycle.rs"]
mod cli_check_rules_postgres_status_lifecycle;
#[path = "cli_check_rules_postgres_table_shape.rs"]
mod cli_check_rules_postgres_table_shape;
#[path = "cli_check_rules_postgres_unmatched_executors.rs"]
mod cli_check_rules_postgres_unmatched_executors;
#[path = "cli_check_rules_postgres_variants.rs"]
mod cli_check_rules_postgres_variants;
#[path = "cli_postgres_catalog.rs"]
mod cli_postgres_catalog;
#[path = "cli_postgres_configured_helper_provenance.rs"]
mod cli_postgres_configured_helper_provenance;
#[path = "cli_postgres_sql_verdicts.rs"]
mod cli_postgres_sql_verdicts;
#[path = "executed_postgres_boolean_api.rs"]
mod executed_postgres_boolean_api;
#[path = "executed_postgres_branch_regressions_api.rs"]
mod executed_postgres_branch_regressions_api;
#[path = "executed_postgres_control_flow_api.rs"]
mod executed_postgres_control_flow_api;
#[path = "executed_postgres_intrinsics_api.rs"]
mod executed_postgres_intrinsics_api;
#[path = "executed_postgres_returned_entrypoints_api.rs"]
mod executed_postgres_returned_entrypoints_api;
#[path = "executed_postgres_source_api.rs"]
mod executed_postgres_source_api;
#[path = "postgres_catalog_real/main.rs"]
mod postgres_catalog_real;
#[path = "postgres_embedded_statement_api.rs"]
mod postgres_embedded_statement_api;

#[path = "cli_check_rules_postgres_test_state.rs"]
mod cli_check_rules_postgres_test_state;
#[path = "cli_check_sql_policy_effort.rs"]
mod cli_check_sql_policy_effort;

#[path = "cli_postgres_snapshot_declared_functions.rs"]
mod cli_postgres_snapshot_declared_functions;

#[path = "cli_postgres_snapshot_executor_recall.rs"]
mod cli_postgres_snapshot_executor_recall;
#[path = "cli_postgres_snapshot_function_locations.rs"]
mod cli_postgres_snapshot_function_locations;

#[path = "cli_postgres_snapshot_join_roots.rs"]
mod cli_postgres_snapshot_join_roots;
#[path = "cli_postgres_snapshot_native_advice.rs"]
mod cli_postgres_snapshot_native_advice;
#[path = "cli_postgres_snapshot_root_proofs.rs"]
mod cli_postgres_snapshot_root_proofs;

#[path = "cli_postgres_executor_names.rs"]
mod cli_postgres_executor_names;
#[path = "cli_postgres_snapshot_fixedpoint.rs"]
mod cli_postgres_snapshot_fixedpoint;

#[path = "cli_postgres_snapshot_empty_before_prefix_source.rs"]
mod cli_postgres_snapshot_empty_before_prefix_source;
#[path = "cli_postgres_snapshot_identifier_aliased_guards.rs"]
mod cli_postgres_snapshot_identifier_aliased_guards;
#[path = "cli_postgres_snapshot_imported_builders.rs"]
mod cli_postgres_snapshot_imported_builders;
#[path = "cli_postgres_snapshot_factory_alias_rejections.rs"]
mod cli_postgres_snapshot_factory_alias_rejections;
#[path = "cli_postgres_snapshot_lock_written_rows.rs"]
mod cli_postgres_snapshot_lock_written_rows;
#[path = "cli_postgres_snapshot_low_executor_shapes.rs"]
mod cli_postgres_snapshot_low_executor_shapes;

#[path = "cli_postgres_snapshot_identifier_compositions.rs"]
mod cli_postgres_snapshot_identifier_compositions;

#[path = "cli_postgres_snapshot_four_joined_lock_aliases.rs"]
mod cli_postgres_snapshot_four_joined_lock_aliases;

#[path = "cli_postgres_transformed_sites.rs"]
mod cli_postgres_transformed_sites;

#[path = "cli_postgres_site_identity.rs"]
mod cli_postgres_site_identity;

#[path = "cli_postgres_snapshot_grouping_recall.rs"]
mod cli_postgres_snapshot_grouping_recall;
#[path = "cli_postgres_snapshot_grouping_review.rs"]
mod cli_postgres_snapshot_grouping_review;

#[path = "cli_postgres_snapshot_grouping_parameter_collision.rs"]
mod cli_postgres_snapshot_grouping_parameter_collision;
#[path = "cli_postgres_snapshot_grouping_recovery.rs"]
mod cli_postgres_snapshot_grouping_recovery;

#[path = "cli_postgres_snapshot_grouping_coverage.rs"]
mod cli_postgres_snapshot_grouping_coverage;

#[path = "cli_postgres_snapshot_issue_3426_regressions.rs"]
mod cli_postgres_snapshot_issue_3426_regressions;

#[path = "cli_postgres_snapshot_issue_3426_transaction.rs"]
mod cli_postgres_snapshot_issue_3426_transaction;
