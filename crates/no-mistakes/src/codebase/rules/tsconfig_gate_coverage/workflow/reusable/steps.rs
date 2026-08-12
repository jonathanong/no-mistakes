use super::ScanContext;
use crate::codebase::ci_graph::triggers::CompiledTriggers;
use crate::codebase::rules::tsconfig_gate_coverage::command_scan;
use serde_yaml::Value;
use std::collections::BTreeSet;

use super::super::{
    conditions::{
        continue_on_error_enabled, step_condition_with_status, EnvironmentState, InputState,
        StaticBool, StaticValue, StepOutcomes,
    },
    runtime::{effective_shell, shell_failure_enforced, shell_pipefail_enforced},
};
use super::validation::action_step_inputs_valid_for_state;

mod checkout;
mod configuration;
mod failure;
mod local_action;
mod model;
mod projects;
mod working_directory;
use model::run_command;
pub(super) use model::StepScan;
use working_directory::{step_working_directory, working_directory_exists};
use {
    checkout::CheckoutState,
    configuration::{job_runtime, job_working_directory, step_configuration_validity},
};

pub(super) fn scan_job_steps(
    job: &Value,
    triggers: &CompiledTriggers,
    inputs: &InputState,
    environment: &EnvironmentState,
    workflow_cwd: Option<String>,
    workflow_shell: Option<String>,
    context: &ScanContext<'_>,
) -> StepScan {
    let Some(steps) = job.get("steps").and_then(Value::as_sequence) else {
        return StepScan {
            projects: BTreeSet::new(),
            failed: false,
            indeterminate: false,
        };
    };
    let job_cwd = job_working_directory(job, inputs, environment, workflow_cwd);
    let (job_shell, implicit_shell_can_be_windows) = job_runtime(job, inputs, workflow_shell);
    let mut projects = BTreeSet::new();
    let mut success = StaticBool::True;
    let mut failed = false;
    let mut indeterminate = false;
    let mut step_outcomes = StepOutcomes::default();
    let mut checkout = CheckoutState::default();
    for step in steps {
        let environment = environment
            .with_step_outcomes(&step_outcomes)
            .with_step(step, inputs);
        let condition = step_condition_with_status(step, inputs, &environment, success);
        let continue_on_error = continue_on_error_enabled(step, inputs, &environment);
        if condition == StaticBool::False {
            step_outcomes.record(step, StaticValue::String("skipped".to_string()));
            continue;
        }
        if condition == StaticBool::Invalid {
            step_outcomes.record(step, StaticValue::String("failure".to_string()));
            failed = true;
            break;
        }
        match step_configuration_validity(step, inputs, &environment) {
            StaticBool::False => {
                if condition == StaticBool::True {
                    step_outcomes.record(step, StaticValue::String("failure".to_string()));
                    failed = true;
                } else {
                    indeterminate = true;
                }
                break;
            }
            StaticBool::Invalid | StaticBool::Unknown | StaticBool::TruthyNonBoolean => {
                indeterminate = true;
                break;
            }
            StaticBool::True => {}
        }
        let uses_action = step.get("uses").is_some();
        if continue_on_error && uses_action {
            continue;
        }
        if uses_action && !action_step_inputs_valid_for_state(step, inputs, &environment) {
            if condition == StaticBool::True {
                step_outcomes.record(step, StaticValue::String("failure".to_string()));
            }
            failed |= condition == StaticBool::True;
            break;
        }
        if let Some(available) = local_action::available(step, &checkout, context.local_actions) {
            if !available {
                failure::record_unavailable(
                    step,
                    condition,
                    &mut step_outcomes,
                    &mut failed,
                    &mut indeterminate,
                );
                break;
            }
            continue;
        }
        checkout.observe(step, condition);
        let step_cwd = step_working_directory(step, inputs, &environment, &job_cwd);
        let Some(cwd) = step_cwd else {
            continue;
        };
        if !working_directory_exists(&cwd, context.tracked) {
            failure::record_unavailable(
                step,
                condition,
                &mut step_outcomes,
                &mut failed,
                &mut indeterminate,
            );
            break;
        }
        let Some(run) = run_command(step) else {
            continue;
        };
        let Some(run) =
            super::super::conditions::resolve_static_interpolations(run, inputs, &environment)
        else {
            continue;
        };
        let shell = effective_shell(step, job_shell.clone());
        if shell.is_none() && implicit_shell_can_be_windows {
            continue;
        }
        let shell = match shell {
            Some(shell) => match super::super::conditions::resolve_static_interpolations(
                &shell,
                inputs,
                &environment,
            ) {
                Some(shell) => Some(shell),
                None => continue,
            },
            None => None,
        };
        let Some(failure_enforced) = shell_failure_enforced(shell.as_deref()) else {
            continue;
        };
        let safe_static_shape = command_scan::shell_body_has_safe_static_shape(&run);
        if !safe_static_shape {
            if !continue_on_error {
                indeterminate |= condition != StaticBool::False;
                break;
            }
            continue;
        }
        let pipefail_enforced = shell_pipefail_enforced(shell.as_deref());
        let reachable_run = command_scan::shell_body_before_static_failure(
            &run,
            failure_enforced,
            pipefail_enforced,
        );
        let run_to_scan = reachable_run.as_str();
        if !continue_on_error {
            projects.extend(projects::scan(
                run_to_scan,
                &cwd,
                failure_enforced,
                triggers,
                context,
            ));
        }
        let pipeline_failure = pipefail_enforced
            && command_scan::shell_body_has_static_pipeline_failure(&run, failure_enforced);
        let static_failure = pipeline_failure
            || command_scan::shell_body_has_static_failure_with_initial(&run, failure_enforced);
        if condition == StaticBool::True && static_failure {
            step_outcomes.record(step, StaticValue::String("failure".to_string()));
            if !continue_on_error {
                success = StaticBool::False;
                failed = true;
            }
        } else if condition == StaticBool::True
            && command_scan::shell_body_is_statically_successful(&run)
        {
            step_outcomes.record(step, StaticValue::String("success".to_string()));
        }
    }
    StepScan::new(projects, failed, indeterminate)
}
