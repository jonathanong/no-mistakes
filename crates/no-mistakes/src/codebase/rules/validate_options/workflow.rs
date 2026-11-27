use super::{decode, RuleValidator};
use crate::codebase::rules;

pub(super) const VALIDATORS: &[RuleValidator] = &[
    (
        rules::github_actions_action_timeout_pair::RULE_ID,
        action_timeout_pair,
    ),
    (
        rules::github_actions_composite_step_schema::RULE_ID,
        composite_step_schema,
    ),
    (
        rules::github_actions_job_permissions::RULE_ID,
        decode::<rules::github_actions_job_permissions::Options>,
    ),
    (rules::github_actions_job_timeouts::RULE_ID, job_timeouts),
    (
        rules::github_actions_local_action_checkout::RULE_ID,
        decode::<rules::github_actions_local_action_checkout::Options>,
    ),
    (rules::github_actions_pinned_hash::RULE_ID, pinned_hash),
    (rules::github_actions_step_timeouts::RULE_ID, step_timeouts),
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

use super::checks::{globs, includes, invalid, nonblank, option};
use super::ValidationContext;
use crate::config::v2::schema::RuleDef;
use anyhow::Result;

fn action_timeout_pair(rule: &RuleDef, _: &ValidationContext<'_>) -> Result<()> {
    let options: rules::github_actions_action_timeout_pair::Options = rule.strict_rule_options()?;
    includes(&rule.rule, &options.include)?;
    for (index, value) in options.uses.iter().enumerate() {
        nonblank(&rule.rule, option("uses").index(index), value)?;
    }
    if options.nested_timeout_seconds.is_some() || options.nested_below_step_timeout {
        nonblank(&rule.rule, option("nestedInput"), &options.nested_input)?;
    }
    Ok(())
}

fn composite_step_schema(rule: &RuleDef, _: &ValidationContext<'_>) -> Result<()> {
    let options: rules::github_actions_composite_step_schema::Options =
        rule.strict_rule_options()?;
    includes(&rule.rule, &options.include)
}

fn pinned_hash(rule: &RuleDef, _: &ValidationContext<'_>) -> Result<()> {
    let options: rules::github_actions_pinned_hash::Options = rule.strict_rule_options()?;
    globs(
        &rule.rule,
        "excludePaths",
        &options.exclude_paths,
        |pattern| pattern,
    )
}

fn job_timeouts(rule: &RuleDef, _: &ValidationContext<'_>) -> Result<()> {
    let options: rules::github_actions_job_timeouts::Options = rule.strict_rule_options()?;
    if options.max_minutes == Some(0) {
        return Err(invalid(
            &rule.rule,
            option("maxMinutes"),
            "must be positive",
        ));
    }
    for (index, entry) in options.allow.iter().enumerate() {
        let path = option("allow").index(index);
        if !entry
            .job
            .rsplit_once('#')
            .is_some_and(|(file, job)| !file.trim().is_empty() && !job.trim().is_empty())
        {
            return Err(invalid(
                &rule.rule,
                path.key("job"),
                "must select a job as workflow-path#job-id",
            ));
        }
        if entry.max_minutes == Some(0) {
            return Err(invalid(
                &rule.rule,
                path.key("maxMinutes"),
                "must be positive",
            ));
        }
    }
    Ok(())
}

fn step_timeouts(rule: &RuleDef, _: &ValidationContext<'_>) -> Result<()> {
    let options: rules::github_actions_step_timeouts::Options = rule.strict_rule_options()?;
    options.validate_minutes()
}
