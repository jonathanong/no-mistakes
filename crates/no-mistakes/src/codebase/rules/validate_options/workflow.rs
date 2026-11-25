use crate::codebase::rules;
use crate::config::v2::schema::RuleDef;
use anyhow::Result;

pub(super) fn workflow(rule: &RuleDef) -> Result<bool> {
    match rule.rule.as_str() {
        rules::github_actions_action_timeout_pair::RULE_ID => {
            rule.strict_rule_options::<rules::github_actions_action_timeout_pair::Options>()?;
        }
        rules::github_actions_checkout_credentials::RULE_ID => {
            rule.strict_rule_options::<rules::github_actions_checkout_credentials::Options>()?;
        }
        rules::github_actions_composite_step_schema::RULE_ID => {
            rule.strict_rule_options::<rules::github_actions_composite_step_schema::Options>()?;
        }
        rules::github_actions_job_permissions::RULE_ID => {
            rule.strict_rule_options::<rules::github_actions_job_permissions::Options>()?;
        }
        rules::github_actions_job_timeouts::RULE_ID => {
            rule.strict_rule_options::<rules::github_actions_job_timeouts::Options>()?;
        }
        rules::github_actions_local_action_checkout::RULE_ID => {
            rule.strict_rule_options::<rules::github_actions_local_action_checkout::Options>()?;
        }
        rules::github_actions_pinned_hash::RULE_ID => {
            rule.strict_rule_options::<rules::github_actions_pinned_hash::Options>()?;
        }
        rules::github_actions_step_timeouts::RULE_ID => {
            rule.strict_rule_options::<rules::github_actions_step_timeouts::Options>()?;
        }
        rules::vitest_ci_path_coverage::RULE_ID => {
            rule.strict_rule_options::<rules::vitest_ci_path_coverage::Options>()?;
        }
        rules::vitest_project_mapping::RULE_ID => {
            rule.strict_rule_options::<rules::vitest_project_mapping::Options>()?;
        }
        rules::vitest_test_correspondence::RULE_ID => {
            rule.strict_rule_options::<rules::vitest_test_correspondence::Options>()?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
