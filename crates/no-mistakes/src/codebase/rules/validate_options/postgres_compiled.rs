use super::RuleValidator;
use crate::codebase::rules;

pub(super) const VALIDATORS: &[RuleValidator] = &[
    (rules::postgres_array_columns::RULE_ID, |rule, _| {
        rules::postgres_array_columns::compile::compile(
            &rule.strict_rule_options()?,
            rule.message.clone(),
        )
        .map(drop)
    }),
    (rules::postgres_bounded_statements::RULE_ID, |rule, _| {
        rules::postgres_bounded_statements::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (
        rules::postgres_column_check_constraints::RULE_ID,
        |rule, _| {
            rules::postgres_column_check_constraints::compile::compile(
                &rule.strict_rule_options()?,
                rule.message.clone(),
            )
            .map(drop)
        },
    ),
    (rules::postgres_column_naming::RULE_ID, |rule, _| {
        rules::postgres_column_naming::validate_options(
            &rule.strict_rule_options()?,
            rule.message.clone(),
        )
        .map(drop)
    }),
    (
        rules::postgres_column_requires_trigger::RULE_ID,
        |rule, _| {
            rules::postgres_column_requires_trigger::compile_options(&rule.strict_rule_options()?)
                .map(drop)
        },
    ),
    (rules::postgres_conflict_ordering::RULE_ID, |rule, _| {
        rules::postgres_conflict_ordering::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (
        rules::postgres_duplicate_function_body::RULE_ID,
        |rule, _| {
            rules::postgres_duplicate_function_body::compile_options(
                &rule.strict_rule_options()?,
                rule.message.clone(),
            )
            .map(drop)
        },
    ),
    (
        rules::postgres_explain_write_rollback::RULE_ID,
        |rule, _| {
            rules::postgres_explain_write_rollback::validate_options(&rule.strict_rule_options()?)
                .map(drop)
        },
    ),
    (rules::postgres_explicit_columns::RULE_ID, |rule, _| {
        rules::postgres_explicit_columns::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::postgres_finite_text_columns::RULE_ID, |rule, _| {
        rules::postgres_finite_text_columns::compile::compile(
            &rule.strict_rule_options()?,
            rule.message.clone(),
        )
        .map(drop)
    }),
    (
        rules::postgres_generated_column_predicates::RULE_ID,
        |rule, _| {
            rules::postgres_generated_column_predicates::compile::compile_options(
                &rule.strict_rule_options()?,
            )
            .map(drop)
        },
    ),
    (
        rules::postgres_grouping_not_aggregation::RULE_ID,
        |rule, _| {
            rules::postgres_grouping_not_aggregation::compile::options(&rule.strict_rule_options()?)
                .map(drop)
        },
    ),
    (rules::postgres_idempotent_insert::RULE_ID, |rule, _| {
        rules::postgres_idempotent_insert::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::postgres_identifier_length::RULE_ID, |rule, _| {
        rules::postgres_identifier_length::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::postgres_json_relation_ids::RULE_ID, |rule, _| {
        rules::postgres_json_relation_ids::compile::compile_options(&rule.strict_rule_options()?)
            .map(drop)
    }),
    (rules::postgres_key_column_types::RULE_ID, |rule, _| {
        rules::postgres_key_column_types::compile::compile(
            &rule.strict_rule_options()?,
            rule.message.clone(),
        )
        .map(drop)
    }),
    (rules::postgres_lock_inventory::RULE_ID, |rule, _| {
        rules::postgres_lock_inventory::options::CompiledOptions::from_options(
            rule.strict_rule_options()?,
        )
        .map(drop)
    }),
    (rules::postgres_lock_ordering::RULE_ID, |rule, _| {
        rules::postgres_lock_ordering::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (
        rules::postgres_lock_written_rows_only::RULE_ID,
        |rule, _| {
            rules::postgres_lock_written_rows_only::compile(&rule.strict_rule_options()?).map(drop)
        },
    ),
    (
        rules::postgres_materialized_view_shape::RULE_ID,
        |rule, _| {
            rules::postgres_materialized_view_shape::compile(&rule.strict_rule_options()?).map(drop)
        },
    ),
    (rules::postgres_polymorphic_target::RULE_ID, |rule, _| {
        rules::postgres_polymorphic_target::compile::compile(
            &rule.strict_rule_options()?,
            rule.message.clone(),
        )
        .map(drop)
    }),
];
