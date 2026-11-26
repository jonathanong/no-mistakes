use super::{decode, RuleValidator};
use crate::codebase::rules;

pub(super) const VALIDATORS: &[RuleValidator] = &[
    (
        rules::github_actions_action_timeout_pair::RULE_ID,
        decode::<rules::github_actions_action_timeout_pair::Options>,
    ),
    (
        rules::github_actions_composite_step_schema::RULE_ID,
        decode::<rules::github_actions_composite_step_schema::Options>,
    ),
    (
        rules::github_actions_job_permissions::RULE_ID,
        decode::<rules::github_actions_job_permissions::Options>,
    ),
    (
        rules::github_actions_job_timeouts::RULE_ID,
        decode::<rules::github_actions_job_timeouts::Options>,
    ),
    (
        rules::github_actions_local_action_checkout::RULE_ID,
        decode::<rules::github_actions_local_action_checkout::Options>,
    ),
    (
        rules::github_actions_pinned_hash::RULE_ID,
        decode::<rules::github_actions_pinned_hash::Options>,
    ),
    (
        rules::github_actions_step_timeouts::RULE_ID,
        decode::<rules::github_actions_step_timeouts::Options>,
    ),
    (
        rules::vitest_ci_path_coverage::RULE_ID,
        decode::<rules::vitest_ci_path_coverage::Options>,
    ),
    (
        rules::vitest_project_mapping::RULE_ID,
        decode::<rules::vitest_project_mapping::Options>,
    ),
    (
        rules::vitest_test_correspondence::RULE_ID,
        decode::<rules::vitest_test_correspondence::Options>,
    ),
];
