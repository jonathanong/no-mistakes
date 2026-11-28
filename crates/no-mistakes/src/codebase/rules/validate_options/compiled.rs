use super::RuleValidator;
use crate::codebase::rules;

pub(super) const VALIDATORS: &[RuleValidator] = &[
    (rules::csharp_no_async_void_delegate::RULE_ID, |rule, _| {
        rules::csharp_no_async_void_delegate::compile_options(rule.strict_rule_options()?).map(drop)
    }),
    (
        rules::github_actions_cache_setup_policy::RULE_ID,
        |rule, _| {
            rules::github_actions_cache_setup_policy::validate_options(rule.strict_rule_options()?)
        },
    ),
    (
        rules::github_actions_download_integrity::RULE_ID,
        |rule, _| {
            rules::github_actions_download_integrity::compile_hosts(&rule.strict_rule_options()?)
                .map(drop)
        },
    ),
    (rules::GITHUB_ACTIONS_ENV_POLICY, |rule, _| {
        rules::github_actions_workflow_files::env_policy::compile(&rule.strict_rule_options()?)
            .map(drop)
    }),
    (rules::GITHUB_ACTIONS_NO_POLL_WAIT_JOB, |rule, _| {
        rules::github_actions_workflow_files::no_poll_wait_job::compile(
            &rule.strict_rule_options()?,
        )
        .map(drop)
    }),
    (rules::GITHUB_ACTIONS_PR_DIFF_RANGE, |rule, _| {
        rules::github_actions_workflow_files::pr_diff_range::validate_options(
            &rule.strict_rule_options()?,
        )
    }),
    (rules::GITHUB_ACTIONS_RUN_COMMAND_POLICY, |rule, _| {
        rules::filesystem_dispatch::github_actions_run_command_policy::options::compile(
            rule.strict_rule_options()?,
        )
        .map(drop)
    }),
    (
        rules::github_actions_script_expressions::RULE_ID,
        |rule, _| {
            rules::github_actions_script_expressions::compile_patterns(&rule.strict_rule_options()?)
                .map(drop)
        },
    ),
    (
        rules::github_actions_test_timeout_literals::RULE_ID,
        |rule, _| {
            rules::github_actions_test_timeout_literals::validate_options(
                &rule.strict_rule_options()?,
            )
            .map(drop)
        },
    ),
    (
        rules::github_actions_vitest_project_selection::RULE_ID,
        |rule, _| {
            rules::github_actions_vitest_project_selection::options::compile(
                &rule.strict_rule_options()?,
            )
            .map(drop)
        },
    ),
    (rules::markdown_child_links::RULE_ID, |rule, _| {
        rules::markdown_child_links::compile_groups(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::no_raw_ephemeral_port::RULE_ID, |rule, _| {
        rules::no_raw_ephemeral_port::compile_options(rule.strict_rule_options()?).map(drop)
    }),
    (rules::no_single_enqueue_in_loop::RULE_ID, |rule, _| {
        rules::no_single_enqueue_in_loop::options::validate(&rule.strict_rule_options()?).map(drop)
    }),
    (rules::no_sparse_checkout::RULE_ID, |rule, _| {
        rules::no_sparse_checkout::compile_options(rule.strict_rule_options()?).map(drop)
    }),
    (rules::no_test_git_sha::RULE_ID, |rule, _| {
        rules::no_test_git_sha::compile_options(rule.strict_rule_options()?).map(drop)
    }),
    (
        rules::queue_no_enqueue_in_transaction::RULE_ID,
        |rule, _| rules::queue_no_enqueue_in_transaction::validate_options(rule),
    ),
    (rules::swift_no_raw_print::RULE_ID, |rule, _| {
        rules::swift_no_raw_print::compile_options(rule.strict_rule_options()?).map(drop)
    }),
    (
        rules::swift_no_unguarded_darwin_import::RULE_ID,
        |rule, _| {
            rules::swift_no_unguarded_darwin_import::compile_options(rule.strict_rule_options()?)
                .map(drop)
        },
    ),
    (rules::swift_viewmodel_main_actor::RULE_ID, |rule, _| {
        rules::swift_viewmodel_main_actor::compile_options(rule.strict_rule_options()?).map(drop)
    }),
    (
        rules::github_actions_max_inline_script_lines::RULE_ID,
        |rule, _| {
            rules::github_actions_max_inline_script_lines::validate_options(
                &rule.strict_rule_options()?,
            )
            .map(drop)
        },
    ),
    (rules::markdown_reachability::RULE_ID, |rule, _| {
        rules::markdown_reachability::validate_options(&rule.strict_rule_options()?).map(drop)
    }),
];
