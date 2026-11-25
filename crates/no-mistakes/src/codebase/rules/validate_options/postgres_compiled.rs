use crate::codebase::rules;
use crate::config::v2::schema::RuleDef;
use anyhow::Result;

pub(super) fn postgres_compiled(rule: &RuleDef) -> Result<bool> {
    match rule.rule.as_str() {
        rules::postgres_array_columns::RULE_ID => {
            rules::postgres_array_columns::compile::compile(
                &rule.strict_rule_options::<rules::postgres_array_columns::Options>()?,
                rule.message.clone(),
            )?;
        }
        rules::postgres_bounded_statements::RULE_ID => {
            rules::postgres_bounded_statements::compile_options(
                &rule.strict_rule_options::<rules::postgres_bounded_statements::Options>()?,
            )?;
        }
        rules::postgres_column_check_constraints::RULE_ID => {
            rules::postgres_column_check_constraints::compile::compile(&rule.strict_rule_options::<crate::config::v2::schema::PostgresColumnCheckConstraintsOptions>()?, rule.message.clone())?;
        }
        rules::postgres_column_naming::RULE_ID => {
            rules::postgres_column_naming::validate_options(
                &rule.strict_rule_options::<rules::postgres_column_naming::Options>()?,
                rule.message.clone(),
            )?;
        }
        rules::postgres_column_requires_trigger::RULE_ID => {
            rules::postgres_column_requires_trigger::compile_options(
                &rule.strict_rule_options::<rules::postgres_column_requires_trigger::Options>()?,
            )?;
        }
        rules::postgres_conflict_ordering::RULE_ID => {
            rules::postgres_conflict_ordering::compile_options(
                &rule.strict_rule_options::<rules::postgres_conflict_ordering::Options>()?,
            )?;
        }
        rules::postgres_duplicate_function_body::RULE_ID => {
            rules::postgres_duplicate_function_body::compile_options(
                &rule.strict_rule_options::<rules::postgres_duplicate_function_body::Options>()?,
                rule.message.clone(),
            )?;
        }
        rules::postgres_explicit_columns::RULE_ID => {
            rules::postgres_explicit_columns::compile_options(
                &rule.strict_rule_options::<rules::postgres_explicit_columns::Options>()?,
            )?;
        }
        rules::postgres_finite_text_columns::RULE_ID => {
            rules::postgres_finite_text_columns::compile::compile(
                &rule.strict_rule_options::<rules::postgres_finite_text_columns::Options>()?,
                rule.message.clone(),
            )?;
        }
        rules::postgres_generated_column_predicates::RULE_ID => {
            rules::postgres_generated_column_predicates::compile::compile_options(
                &rule.strict_rule_options::<rules::postgres_generated_column_predicates::Options>(
                )?,
            )?;
        }
        rules::postgres_grouping_not_aggregation::RULE_ID => {
            rules::postgres_grouping_not_aggregation::compile::options(
                &rule.strict_rule_options::<rules::postgres_grouping_not_aggregation::Options>()?,
            )?;
        }
        rules::postgres_idempotent_insert::RULE_ID => {
            rules::postgres_idempotent_insert::compile_options(
                &rule
                    .strict_rule_options::<rules::postgres_idempotent_insert::options::Options>()?,
            )?;
        }
        rules::postgres_identifier_length::RULE_ID => {
            rules::postgres_identifier_length::compile_options(
                &rule.strict_rule_options::<rules::postgres_identifier_length::Options>()?,
            )?;
        }
        rules::postgres_json_relation_ids::RULE_ID => {
            rules::postgres_json_relation_ids::compile::compile_options(
                &rule.strict_rule_options::<rules::postgres_json_relation_ids::Options>()?,
            )?;
        }
        rules::postgres_key_column_types::RULE_ID => {
            rules::postgres_key_column_types::compile::compile(
                &rule.strict_rule_options::<rules::postgres_key_column_types::Options>()?,
                rule.message.clone(),
            )?;
        }
        rules::postgres_lock_inventory::RULE_ID => {
            rules::postgres_lock_inventory::options::CompiledOptions::from_options(
                rule.strict_rule_options::<rules::postgres_lock_inventory::options::Options>()?,
            )?;
        }
        rules::postgres_lock_ordering::RULE_ID => {
            rules::postgres_lock_ordering::compile_options(
                &rule.strict_rule_options::<rules::postgres_lock_ordering::Options>()?,
            )?;
        }
        rules::postgres_lock_written_rows_only::RULE_ID => {
            rules::postgres_lock_written_rows_only::compile(
                &rule.strict_rule_options::<rules::postgres_lock_written_rows_only::Options>()?,
            )?;
        }
        rules::postgres_materialized_view_shape::RULE_ID => {
            rules::postgres_materialized_view_shape::compile(
                &rule.strict_rule_options::<rules::postgres_materialized_view_shape::Options>()?,
            )?;
        }
        rules::postgres_no_generated_column_writes::RULE_ID => {
            rules::postgres_no_generated_column_writes::compile_options(
                &rule
                    .strict_rule_options::<rules::postgres_no_generated_column_writes::Options>()?,
            )?;
        }
        rules::postgres_no_mutable_sql_registry::RULE_ID => {
            rules::postgres_sql_cross_file::compile(
                &rule.rule,
                &rule.strict_rule_options::<rules::postgres_sql_cross_file::Options>()?,
            )?;
        }
        rules::postgres_no_offset::RULE_ID => {
            rules::postgres_no_offset::compile_options(
                &rule.strict_rule_options::<rules::postgres_no_offset::Options>()?,
            )?;
        }
        rules::postgres_no_test_state_mutation::RULE_ID => {
            rules::postgres_no_test_state_mutation::CompiledOptions::new(
                &rule.strict_rule_options::<rules::postgres_no_test_state_mutation::Options>()?,
            )?;
        }
        rules::postgres_no_transformed_uuid_predicates::RULE_ID => {
            rules::postgres_no_transformed_uuid_predicates::compile_options(
                &rule
                    .strict_rule_options::<rules::postgres_no_transformed_uuid_predicates::Options>(
                    )?,
            )?;
        }
        rules::postgres_no_unbounded_sql_identifier::RULE_ID => {
            rules::postgres_sql_cross_file::compile(
                &rule.rule,
                &rule.strict_rule_options::<rules::postgres_sql_cross_file::Options>()?,
            )?;
        }
        rules::postgres_object_naming::RULE_ID => {
            rules::postgres_object_naming::compile::compile(
                &rule.strict_rule_options::<rules::postgres_object_naming::Options>()?,
                rule.message.clone(),
            )?;
        }
        rules::postgres_required_comments::RULE_ID => {
            rules::postgres_required_comments::compile_options(
                &rule.strict_rule_options::<rules::postgres_required_comments::Options>()?,
            )?;
        }
        rules::postgres_required_predicates::RULE_ID => {
            rules::postgres_required_predicates::compile_options(
                &rule.strict_rule_options::<rules::postgres_required_predicates::Options>()?,
            )?;
        }
        rules::postgres_require_query_annotation::RULE_ID => {
            rules::postgres_require_query_annotation::compile_options(
                &rule.strict_rule_options::<rules::postgres_require_query_annotation::Options>()?,
            )?;
        }
        rules::postgres_sql_shape_policy::RULE_ID => {
            rules::postgres_sql_shape_policy::compile_options(
                &rule.strict_rule_options::<rules::postgres_sql_shape_policy::Options>()?,
            )?;
        }
        rules::postgres_sql_statement_policy::RULE_ID => {
            rules::postgres_sql_statement_policy::compile_options(
                &rule
                    .strict_rule_options::<rules::postgres_sql_statement_policy::options::Options>(
                    )?,
            )?;
        }
        rules::postgres_status_with_lifecycle_timestamps::RULE_ID => {
            rules::postgres_status_with_lifecycle_timestamps::compile_options(&rule.strict_rule_options::<rules::postgres_status_with_lifecycle_timestamps::Options>()?, rule.message.clone())?;
        }
        rules::postgres_table_shape::RULE_ID => {
            rules::postgres_table_shape::compile::compile(
                &rule.strict_rule_options::<rules::postgres_table_shape::Options>()?,
                rule.message.clone(),
            )?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
