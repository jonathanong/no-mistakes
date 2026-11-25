use crate::codebase::rules;
use crate::config::v2::schema::RuleDef;
use anyhow::Result;
use std::path::Path;

pub(super) fn validate(rule: &RuleDef, root: &Path) -> Result<bool> {
    match rule.rule.as_str() {
        rules::DEPENDABOT_COOLDOWN_POLICY => {
            rules::filesystem_dispatch::dependabot_cooldown_policy::compile_options(
                &rule.strict_rule_options()?,
            )?;
        }
        rules::github_actions_checkout_credentials::RULE_ID => {
            rule.strict_rule_options::<rules::github_actions_checkout_credentials::Options>()?
                .default_branch()?;
        }
        rules::CSHARP_MAX_LINES_PER_FILE => {
            rules::csharp_max_lines_per_file::prepared::PreparedRule::new(
                &rule.strict_rule_options()?,
                root,
                &[],
            )?;
        }
        rules::REQUIRE_STORYBOOK_STORIES => {
            rules::require_storybook_stories::types::validate_options(
                &rule.strict_rule_options()?,
            )?;
        }
        rules::REQUIRED_COMPANION_IMPORTS => {
            rules::required_companion_imports::validate_options(&rule.strict_rule_options()?)?;
        }
        rules::REQUIRED_DOC_SECTION => {
            rules::required_local_docs::doc_section::compile_options(&rule.strict_rule_options()?)?;
        }
        rules::UNCONSTRUCTED_ERROR_CLASS => {
            rule.strict_rule_options::<rules::unconstructed_error_class::config::Options>()?
                .test_file_matcher()?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
