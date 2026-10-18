//! Cli postgres integration tests, linked in one harness.

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
#[path = "cli_check_rules_postgres_required_comments.rs"]
mod cli_check_rules_postgres_required_comments;
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
