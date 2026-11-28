use super::{RuleDef, RuleValidator, ValidationContext};
use crate::codebase::rules;

pub(super) const VALIDATORS: &[RuleValidator] = &[
    (
        rules::postgres_no_duplicate_keyed_lookups::RULE_ID,
        |rule, _| {
            rules::postgres_no_duplicate_keyed_lookups::compile_options(
                &rule.strict_rule_options()?,
            )
            .map(drop)
        },
    ),
    (
        rules::postgres_no_generated_column_writes::RULE_ID,
        |rule, _| {
            rules::postgres_no_generated_column_writes::compile_options(
                &rule.strict_rule_options()?,
            )
            .map(drop)
        },
    ),
    (
        rules::postgres_no_mutable_sql_registry::RULE_ID,
        cross_file_options,
    ),
    (rules::postgres_no_offset::RULE_ID, |rule, _| {
        rules::postgres_no_offset::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (
        rules::postgres_no_test_state_mutation::RULE_ID,
        |rule, _| {
            rules::postgres_no_test_state_mutation::CompiledOptions::new(
                &rule.strict_rule_options()?,
            )
            .map(drop)
        },
    ),
    (
        rules::postgres_no_transformed_uuid_predicates::RULE_ID,
        |rule, _| {
            rules::postgres_no_transformed_uuid_predicates::compile_options(
                &rule.strict_rule_options()?,
            )
            .map(drop)
        },
    ),
    (
        rules::postgres_no_unbounded_sql_identifier::RULE_ID,
        cross_file_options,
    ),
    (rules::postgres_object_naming::RULE_ID, |rule, _| {
        rules::postgres_object_naming::compile::compile(
            &rule.strict_rule_options()?,
            rule.message.clone(),
        )
        .map(drop)
    }),
    (rules::postgres_required_comments::RULE_ID, |rule, _| {
        rules::postgres_required_comments::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::postgres_required_predicates::RULE_ID, |rule, _| {
        rules::postgres_required_predicates::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (
        rules::postgres_require_query_annotation::RULE_ID,
        |rule, _| {
            rules::postgres_require_query_annotation::compile_options(&rule.strict_rule_options()?)
                .map(drop)
        },
    ),
    (rules::postgres_sql_shape_policy::RULE_ID, |rule, _| {
        rules::postgres_sql_shape_policy::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::postgres_sql_statement_policy::RULE_ID, |rule, _| {
        rules::postgres_sql_statement_policy::compile_options(&rule.strict_rule_options()?)
            .map(drop)
    }),
    (
        rules::postgres_status_with_lifecycle_timestamps::RULE_ID,
        |rule, _| {
            rules::postgres_status_with_lifecycle_timestamps::compile_options(
                &rule.strict_rule_options()?,
                rule.message.clone(),
            )
            .map(drop)
        },
    ),
    (rules::postgres_table_shape::RULE_ID, |rule, _| {
        rules::postgres_table_shape::compile::compile(
            &rule.strict_rule_options()?,
            rule.message.clone(),
        )
        .map(drop)
    }),
];

fn cross_file_options(rule: &RuleDef, _context: &ValidationContext<'_>) -> anyhow::Result<()> {
    // Project materialization owns this key and rejects authored copies of it.
    if rule.options.get("sqlIncludeLocked").is_none() {
        crate::config::v2::validate_low_alias(rule)?;
    }
    rules::postgres_sql_cross_file::compile(&rule.rule, &rule.strict_rule_options()?).map(drop)
}
