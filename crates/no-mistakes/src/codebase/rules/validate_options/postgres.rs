use crate::codebase::rules;
use crate::config::v2::schema::RuleDef;
use anyhow::Result;

pub(super) fn postgres(rule: &RuleDef) -> Result<bool> {
    match rule.rule.as_str() {
        rules::postgres_constraint_validate::RULE_ID => {
            rule.strict_rule_options::<rules::simple_schema_rule::Options>()?;
        }
        rules::postgres_fk_index::RULE_ID => {
            rule.strict_rule_options::<rules::postgres_fk_index::Options>()?;
        }
        rules::postgres_no_add_column::RULE_ID => {
            rule.strict_rule_options::<rules::postgres_no_add_column::Options>()?;
        }
        rules::postgres_redundant_index::RULE_ID => {
            rule.strict_rule_options::<rules::postgres_redundant_index::Options>()?;
        }
        rules::postgres_require_fk_on_delete::RULE_ID => {
            rule.strict_rule_options::<rules::simple_schema_rule::Options>()?;
        }
        rules::postgres_require_named_constraints::RULE_ID => {
            rule.strict_rule_options::<rules::simple_schema_rule::Options>()?;
        }
        rules::postgres_sql_fragment_escape::RULE_ID => {
            rule.strict_rule_options::<rules::postgres_sql_fragment_escape::Options>()?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
