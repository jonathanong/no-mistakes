use super::RuleValidator;
use crate::codebase::rules;

pub(super) const VALIDATORS: &[RuleValidator] = &[
    (rules::DEPENDABOT_COOLDOWN_POLICY, |rule, _| {
        rules::dependabot_cooldown_policy::compile_options(&rule.strict_rule_options()?).map(drop)
    }),
    (
        rules::github_actions_checkout_credentials::RULE_ID,
        |rule, _| {
            rule.strict_rule_options::<rules::github_actions_checkout_credentials::Options>()?
                .default_branch()
                .map(drop)
        },
    ),
    (rules::CSHARP_MAX_LINES_PER_FILE, |rule, context| {
        rules::csharp_max_lines_per_file::prepared::PreparedRule::new(
            &rule.strict_rule_options()?,
            context.root,
            &[],
        )
        .map(drop)
    }),
    (rules::REQUIRE_STORYBOOK_STORIES, |rule, _| {
        rules::require_storybook_stories::types::validate_options(&rule.strict_rule_options()?)
            .map(drop)
    }),
    (rules::REQUIRED_COMPANION_IMPORTS, |rule, _| {
        rules::required_companion_imports::validate_options(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::REQUIRED_DOC_SECTION, |rule, _| {
        rules::required_local_docs::doc_section::compile_options(&rule.strict_rule_options()?)
            .map(drop)
    }),
    (rules::UNCONSTRUCTED_ERROR_CLASS, |rule, _| {
        rule.strict_rule_options::<rules::unconstructed_error_class::config::Options>()?
            .test_file_matcher()
            .map(drop)
    }),
];
