use crate::codebase::rules;
use crate::config::v2::schema::RuleDef;
use anyhow::Result;

pub(super) fn compiled(rule: &RuleDef) -> Result<bool> {
    match rule.rule.as_str() {
        rules::csharp_no_async_void_delegate::RULE_ID => {
            rules::csharp_no_async_void_delegate::compile_options(
                rule.strict_rule_options::<rules::csharp_no_async_void_delegate::Options>()?,
            )?;
        }
        rules::github_actions_download_integrity::RULE_ID => {
            rules::github_actions_download_integrity::compile_hosts(
                &rule.strict_rule_options::<rules::github_actions_download_integrity::Options>()?,
            )?;
        }
        rules::GITHUB_ACTIONS_ENV_POLICY => {
            rules::github_actions_workflow_files::env_policy::compile(&rule.strict_rule_options::<rules::github_actions_workflow_files::env_policy::Options>()?)?;
        }
        rules::GITHUB_ACTIONS_NO_POLL_WAIT_JOB => {
            rules::github_actions_workflow_files::no_poll_wait_job::compile(&rule.strict_rule_options::<rules::github_actions_workflow_files::no_poll_wait_job::Options>()?)?;
        }
        rules::GITHUB_ACTIONS_RUN_COMMAND_POLICY => {
            rules::filesystem_dispatch::github_actions_run_command_policy::options::compile(rule.strict_rule_options::<rules::filesystem_dispatch::github_actions_run_command_policy::options::Options>()?)?;
        }
        rules::github_actions_script_expressions::RULE_ID => {
            rules::github_actions_script_expressions::compile_patterns(
                &rule.strict_rule_options::<rules::github_actions_script_expressions::Options>()?,
            )?;
        }
        rules::github_actions_test_timeout_literals::RULE_ID => {
            rules::github_actions_test_timeout_literals::validate_options(
                &rule.strict_rule_options::<rules::github_actions_test_timeout_literals::Options>(
                )?,
            )?;
        }
        rules::github_actions_vitest_project_selection::RULE_ID => {
            rules::github_actions_vitest_project_selection::options::compile(
                &rule
                    .strict_rule_options::<rules::github_actions_vitest_project_selection::Options>(
                    )?,
            )?;
        }
        rules::markdown_child_links::RULE_ID => {
            rules::markdown_child_links::compile_groups(
                &rule.strict_rule_options::<rules::markdown_child_links::Options>()?,
            )?;
        }
        rules::no_raw_ephemeral_port::RULE_ID => {
            rules::no_raw_ephemeral_port::compile_options(
                rule.strict_rule_options::<rules::no_raw_ephemeral_port::Options>()?,
            )?;
        }
        rules::no_single_enqueue_in_loop::RULE_ID => {
            rules::no_single_enqueue_in_loop::options::validate(
                &rule
                    .strict_rule_options::<rules::no_single_enqueue_in_loop::options::Options>()?,
            )?;
        }
        rules::no_sparse_checkout::RULE_ID => {
            rules::no_sparse_checkout::compile_options(
                rule.strict_rule_options::<rules::no_sparse_checkout::Options>()?,
            )?;
        }
        rules::no_test_git_sha::RULE_ID => {
            rules::no_test_git_sha::compile_options(
                rule.strict_rule_options::<rules::no_test_git_sha::Options>()?,
            )?;
        }
        rules::swift_no_raw_print::RULE_ID => {
            rules::swift_no_raw_print::compile_options(
                rule.strict_rule_options::<rules::swift_no_raw_print::Options>()?,
            )?;
        }
        rules::swift_viewmodel_main_actor::RULE_ID => {
            rules::swift_viewmodel_main_actor::compile_options(
                rule.strict_rule_options::<rules::swift_viewmodel_main_actor::Options>()?,
            )?;
        }
        rules::github_actions_max_inline_script_lines::RULE_ID => {
            rules::github_actions_max_inline_script_lines::validate_options(
                &rule
                    .strict_rule_options::<rules::github_actions_max_inline_script_lines::Options>(
                    )?,
            )?;
        }
        rules::markdown_reachability::RULE_ID => {
            rules::markdown_reachability::validate_options(
                &rule.strict_rule_options::<rules::markdown_reachability::Options>()?,
            )?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
