use crate::codebase::rules;
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

pub(super) fn compiled_misc(
    rule: &RuleDef,
    config: &NoMistakesConfig,
    root: &Path,
) -> Result<bool> {
    match rule.rule.as_str() {
        rules::banned_paths::RULE_ID => {
            rules::banned_paths::compile_bans(
                &rule.strict_rule_options::<rules::banned_paths::Options>()?,
            )?;
        }
        rules::banned_renamed_files::RULE_ID => {
            rules::banned_renamed_files::matching::compile_basenames(
                &rule.strict_rule_options::<rules::banned_renamed_files::Options>()?,
            )?;
        }
        rules::config_path_references::RULE_ID => {
            rule.strict_rule_options::<rules::config_path_references::Options>()?
                .validate()?;
        }
        rules::coverage_producer_assertion_owner::RULE_ID => {
            rule.strict_rule_options::<rules::coverage_producer_assertion_owner::config::Options>(
            )?
            .validate()?;
        }
        rules::finite_set_consistency::RULE_ID => {
            rule.strict_rule_options::<rules::finite_set_consistency::Options>()?
                .validate()?;
        }
        rules::forbidden_calls::RULE_ID => {
            rules::forbidden_calls::config::validate(
                &rule.strict_rule_options::<rules::forbidden_calls::config::Options>()?,
            )?;
        }
        rules::forbidden_lint_suppressions::RULE_ID => {
            rule.strict_rule_options::<rules::forbidden_lint_suppressions::options::Options>()?
                .validate()?;
        }
        rules::integration_test_no_mocks::RULE_ID => {
            rules::integration_test_no_mocks::compile_options(
                &rule.strict_rule_options::<rules::integration_test_no_mocks::Options>()?,
            )?;
        }
        rules::query_reached_per_item::RULE_ID => {
            rules::query_reached_per_item::config::validate(
                &rule.strict_rule_options::<rules::query_reached_per_item::config::Options>()?,
                config,
            )?;
        }
        rules::test_email_domain_policy::RULE_ID => {
            rules::test_email_domain_policy::compile_options(
                &rule.strict_rule_options::<rules::test_email_domain_policy::Options>()?,
            )?;
        }
        rules::test_fixture_size_cap::RULE_ID => {
            rules::test_fixture_size_cap::compile_options(
                &rule.strict_rule_options::<rules::test_fixture_size_cap::Options>()?,
            )?;
        }
        rules::test_no_dependency_pins::RULE_ID => {
            rules::test_no_dependency_pins::compile_options(
                &rule.strict_rule_options::<rules::test_no_dependency_pins::Options>()?,
            )?;
        }
        rules::test_no_live_repository_scan::RULE_ID => {
            rule.strict_rule_options::<rules::test_no_live_repository_scan::config::Options>()?
                .validate()?;
        }
        rules::vitest_project_policy::RULE_ID => {
            rule.strict_rule_options::<rules::vitest_project_policy::options::Options>()?
                .validate_for_config_load(root)?;
        }
        rules::version_pin_consistency::RULE_ID => {
            rules::version_pin_consistency::validate_options(
                &rule.strict_rule_options::<rules::version_pin_consistency::Options>()?,
            )?;
        }
        rules::workflow_topology_policy::RULE_ID => {
            rules::workflow_topology_policy::validate_options(
                &rule.strict_rule_options::<rules::workflow_topology_policy::Options>()?,
            )?;
        }
        rules::production_exports_test_only::RULE_ID => {
            rules::production_exports_test_only::config::Selection::from_options(
                root,
                rule.strict_rule_options::<rules::production_exports_test_only::config::Options>()?,
            )?;
        }
        rules::doc_consistency::RULE_ID => {
            rules::doc_consistency::options::validate_options(
                &rule.strict_rule_options::<rules::doc_consistency::options::Options>()?,
            )?;
        }
        rules::TEST_NO_RETRIES => {
            rules::test_reliability::options::validate_test_no_retries(
                rule.strict_rule_options::<rules::test_reliability::options::RetryOptions>()?,
            )?;
        }
        rules::PLAYWRIGHT_NO_SERIAL => {
            rules::test_reliability::options::validate_playwright_no_serial(
                rule.strict_rule_options::<rules::test_reliability::options::SerialOptions>()?,
            )?;
        }
        rules::BASH_EMPTY_ARRAY_NOUNSET => {
            rules::filesystem_dispatch::shell_compatibility::validate_options(&rule.rule, &rule.strict_rule_options::<rules::filesystem_dispatch::shell_compatibility::Options>()?)?;
        }
        rules::ZSH_READONLY_PARAMETER => {
            rules::filesystem_dispatch::shell_compatibility::validate_options(&rule.rule, &rule.strict_rule_options::<rules::filesystem_dispatch::shell_compatibility::Options>()?)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
