use super::RuleValidator;
use crate::codebase::rules;

pub(super) const VALIDATORS: &[RuleValidator] = &[
    (rules::banned_paths::RULE_ID, |rule, _| {
        rules::banned_paths::compile_bans(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::banned_renamed_files::RULE_ID, |rule, _| {
        rules::banned_renamed_files::matching::compile_basenames(&rule.strict_rule_options()?)
            .map(drop)
    }),
    (rules::config_path_references::RULE_ID, |rule, _| {
        rule.strict_rule_options::<rules::config_path_references::Options>()?
            .validate()
            .map(drop)
    }),
    (
        rules::coverage_producer_assertion_owner::RULE_ID,
        |rule, _| {
            rule.strict_rule_options::<rules::coverage_producer_assertion_owner::config::Options>()?
                .validate()
                .map(drop)
        },
    ),
    (rules::csp_allowed_origins::RULE_ID, |rule, _| {
        rules::csp_allowed_origins::validate_options(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::finite_set_consistency::RULE_ID, |rule, _| {
        rule.strict_rule_options::<rules::finite_set_consistency::Options>()?
            .validate()
            .map(drop)
    }),
    (rules::forbidden_calls::RULE_ID, |rule, _| {
        rules::forbidden_calls::config::validate(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::forbidden_lint_suppressions::RULE_ID, |rule, _| {
        rule.strict_rule_options::<rules::forbidden_lint_suppressions::options::Options>()?
            .validate()
            .map(drop)
    }),
    (rules::integration_test_no_mocks::RULE_ID, |rule, _| {
        rules::integration_test_no_mocks::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::query_reached_per_item::RULE_ID, |rule, context| {
        rules::query_reached_per_item::config::validate(
            &rule.strict_rule_options()?,
            context.config,
        )
        .map(drop)
    }),
    (rules::test_email_domain_policy::RULE_ID, |rule, _| {
        rules::test_email_domain_policy::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::test_fixture_size_cap::RULE_ID, |rule, _| {
        rules::test_fixture_size_cap::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::test_no_dependency_pins::RULE_ID, |rule, _| {
        rules::test_no_dependency_pins::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::test_no_live_repository_scan::RULE_ID, |rule, _| {
        rule.strict_rule_options::<rules::test_no_live_repository_scan::config::Options>()?
            .validate()
            .map(drop)
    }),
    (rules::vitest_project_policy::RULE_ID, |rule, context| {
        rule.strict_rule_options::<rules::vitest_project_policy::options::Options>()?
            .validate_for_config_load(context.root)
            .map(drop)
    }),
    (rules::version_pin_consistency::RULE_ID, |rule, _| {
        rules::version_pin_consistency::validate_options(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::workflow_topology_policy::RULE_ID, |rule, _| {
        rules::workflow_topology_policy::validate_options(&rule.strict_rule_options()?).map(drop)
    }),
    (
        rules::production_exports_test_only::RULE_ID,
        |rule, context| {
            rules::production_exports_test_only::config::Selection::from_options(
                context.root,
                rule.strict_rule_options()?,
            )
            .map(drop)
        },
    ),
    (rules::doc_consistency::RULE_ID, |rule, _| {
        rules::doc_consistency::options::validate_options(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::TEST_NO_RETRIES, |rule, _| {
        rules::test_reliability::options::validate_test_no_retries(rule.strict_rule_options()?)
            .map(drop)
    }),
    (rules::PLAYWRIGHT_NO_SERIAL, |rule, _| {
        rules::test_reliability::options::validate_playwright_no_serial(rule.strict_rule_options()?)
            .map(drop)
    }),
    (rules::BASH_EMPTY_ARRAY_NOUNSET, |rule, _| {
        rules::filesystem_dispatch::shell_compatibility::validate_options(
            &rule.rule,
            &rule.strict_rule_options()?,
        )
        .map(drop)
    }),
    (rules::NO_CROSS_PACKAGE_RELATIVE_IMPORTS, |rule, _| {
        rules::no_cross_package_relative_imports::accept_declared_options(rule)
    }),
    (rules::ZSH_READONLY_PARAMETER, |rule, _| {
        rules::filesystem_dispatch::shell_compatibility::validate_options(
            &rule.rule,
            &rule.strict_rule_options()?,
        )
        .map(drop)
    }),
];
