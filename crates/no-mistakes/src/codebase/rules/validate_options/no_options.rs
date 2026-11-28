use super::{decode, NoOptions, RuleValidator};
use crate::codebase::rules;
use crate::playwright::rules as playwright;

pub(super) const VALIDATORS: &[RuleValidator] = &[
    (
        rules::github_actions_docker_host_ports::RULE_ID,
        decode::<NoOptions>,
    ),
    (
        rules::GITHUB_ACTIONS_NO_INSTA_SNAPSHOT_UPDATES,
        decode::<NoOptions>,
    ),
    (rules::MARKDOWN_MERMAID_VALIDATION, decode::<NoOptions>),
    (rules::NO_MISTAKES_CONFIG, decode::<NoOptions>),
    (rules::PNPM_OVERRIDES_BAN, decode::<NoOptions>),
    (rules::NEXTJS_NO_API_ROUTES, decode::<NoOptions>),
    (rules::NEXTJS_NO_CACHING, decode::<NoOptions>),
    (
        rules::REACT_NO_NESTED_BUTTONS_AND_LINKS,
        decode::<NoOptions>,
    ),
    (
        rules::TERRAFORM_IAM_NO_UNDOCUMENTED_WILDCARDS,
        decode::<NoOptions>,
    ),
    (rules::TEST_NO_UNMOCKED_DYNAMIC_IMPORTS, decode::<NoOptions>),
    (playwright::PLAYWRIGHT_COVERAGE, decode::<NoOptions>),
    (playwright::PLAYWRIGHT_UNIQUE_TEST_IDS, decode::<NoOptions>),
    (playwright::PLAYWRIGHT_UNIQUE_HTML_IDS, decode::<NoOptions>),
    (
        playwright::PLAYWRIGHT_PREFER_TEST_ID_LOCATORS,
        decode::<NoOptions>,
    ),
    ("assert-no-fetch", decode::<NoOptions>),
];
