use super::ScanContext;
use crate::codebase::ci_graph::triggers::CompiledTriggers;
use serde_yaml::Value;
use std::collections::BTreeSet;

use super::super::conditions::{
    continue_on_error_value, step_condition_with_status, EnvironmentState, InputState, StaticBool,
    StaticValue, StepOutcomes,
};
use super::validation::action_step_inputs_valid_for_state;

mod checkout;
mod configuration;
mod local_action;
mod model;
mod run;
mod working_directory;
pub(super) use model::StepScan;
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
        let continue_on_error = continue_on_error_value(step, inputs, &environment);
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
        let continue_on_error = match continue_on_error {
            StaticBool::True => true,
            StaticBool::False => false,
            // A dynamic tolerance can either hide a failure or stop later gates.
            _ => {
                indeterminate = true;
                break;
            }
        };
        let uses_action = step.get("uses").is_some();
        if uses_action && !action_step_inputs_valid_for_state(step, inputs, &environment) {
            if continue_on_error {
                continue;
            }
            if condition == StaticBool::True {
                step_outcomes.record(step, StaticValue::String("failure".to_string()));
            }
            failed |= condition == StaticBool::True;
            break;
        }
        checkout.observe(step, condition, inputs, &environment);
        if uses_action && condition == StaticBool::True {
            step_outcomes.record(step, StaticValue::String("success".to_string()));
        }
        if continue_on_error && uses_action {
            continue;
        }
        if let Some(available) = local_action::available(step, &checkout, context.local_actions) {
            if !available {
                if condition == StaticBool::True {
                    step_outcomes.record(step, StaticValue::String("failure".to_string()));
                    failed = true;
                } else {
                    indeterminate = true;
                }
                break;
            }
            continue;
        }
        if uses_action {
            continue;
        }
        if run::run_step_stops_job(
            step,
            &run::RunStepConfiguration {
                inputs,
                environment: &environment,
                job_cwd: &job_cwd,
                job_shell: job_shell.clone(),
                implicit_shell_can_be_windows,
                triggers,
                context,
                condition,
                continue_on_error,
            },
            &mut run::RunStepState {
                projects: &mut projects,
                step_outcomes: &mut step_outcomes,
                success: &mut success,
                failed: &mut failed,
                indeterminate: &mut indeterminate,
            },
        ) {
            break;
        }
    }
    StepScan::new(projects, failed, indeterminate)
}
