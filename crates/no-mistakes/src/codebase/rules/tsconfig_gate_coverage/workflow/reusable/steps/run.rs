use super::{model, working_directory};
use crate::codebase::ci_graph::triggers::{CompiledTriggers, TriggerMatch};
use crate::codebase::rules::tsconfig_gate_coverage::{
    application::resolve_gate_project_against_tracked, command_scan,
};
use serde_yaml::Value;
use std::collections::BTreeSet;

use super::super::super::{
    conditions::{EnvironmentState, InputState, StaticBool, StaticValue, StepOutcomes},
    runtime::{effective_shell, shell_failure_enforced, shell_pipefail_enforced},
};
use super::super::ScanContext;

pub(super) struct RunStepConfiguration<'a, 'context> {
    pub(super) inputs: &'a InputState,
    pub(super) environment: &'a EnvironmentState,
    pub(super) job_cwd: &'a Option<String>,
    pub(super) job_shell: Option<String>,
    pub(super) implicit_shell_can_be_windows: bool,
    pub(super) triggers: &'a CompiledTriggers,
    pub(super) context: &'a ScanContext<'context>,
    pub(super) condition: StaticBool,
    pub(super) continue_on_error: bool,
}

pub(super) struct RunStepState<'a> {
    pub(super) projects: &'a mut BTreeSet<String>,
    pub(super) step_outcomes: &'a mut StepOutcomes,
    pub(super) success: &'a mut StaticBool,
    pub(super) failed: &'a mut bool,
    pub(super) indeterminate: &'a mut bool,
}

enum ShellResolution {
    Resolved(Option<String>),
    Unresolved,
    UnsupportedImplicit,
}

fn resolved_shell(step: &Value, configuration: &RunStepConfiguration<'_, '_>) -> ShellResolution {
    let shell = effective_shell(step, configuration.job_shell.clone());
    if shell.is_none() && configuration.implicit_shell_can_be_windows {
        return ShellResolution::UnsupportedImplicit;
    }
    match shell {
        Some(shell) => super::super::super::conditions::resolve_static_interpolations(
            &shell,
            configuration.inputs,
            configuration.environment,
        )
        .map(|shell| ShellResolution::Resolved(Some(shell)))
        .unwrap_or(ShellResolution::Unresolved),
        None => ShellResolution::Resolved(None),
    }
}

pub(super) fn run_step_stops_job(
    step: &Value,
    configuration: &RunStepConfiguration<'_, '_>,
    state: &mut RunStepState<'_>,
) -> bool {
    let Some(cwd) = working_directory::step_working_directory(
        step,
        configuration.inputs,
        configuration.environment,
        configuration.job_cwd,
    ) else {
        if !configuration.continue_on_error {
            *state.indeterminate |= configuration.condition != StaticBool::False;
            return true;
        }
        return false;
    };
    if !working_directory::working_directory_exists(&cwd, &configuration.context.visible_paths) {
        if configuration.condition == StaticBool::True {
            state.step_outcomes.record_with_conclusion(
                step,
                StaticValue::String("failure".to_string()),
                StaticValue::String(
                    if configuration.continue_on_error {
                        "success"
                    } else {
                        "failure"
                    }
                    .to_string(),
                ),
            );
        }
        if configuration.continue_on_error {
            return false;
        }
        *state.failed |= configuration.condition == StaticBool::True;
        *state.indeterminate |= configuration.condition != StaticBool::True;
        return true;
    }
    let Some(run) = model::run_command(step) else {
        return false;
    };
    let Some(run) = super::super::super::conditions::resolve_static_interpolations(
        run,
        configuration.inputs,
        configuration.environment,
    ) else {
        if !configuration.continue_on_error {
            *state.indeterminate |= configuration.condition != StaticBool::False;
            return true;
        }
        return false;
    };
    let shell = match resolved_shell(step, configuration) {
        ShellResolution::Resolved(shell) => shell,
        ShellResolution::UnsupportedImplicit => {
            if !configuration.continue_on_error {
                *state.indeterminate |= configuration.condition != StaticBool::False;
                return true;
            }
            return false;
        }
        ShellResolution::Unresolved => {
            if !configuration.continue_on_error {
                *state.indeterminate |= configuration.condition != StaticBool::False;
                return true;
            }
            return false;
        }
    };
    let Some(failure_enforced) = shell_failure_enforced(shell.as_deref()) else {
        if !configuration.continue_on_error && configuration.condition != StaticBool::False {
            *state.indeterminate = true;
            return true;
        }
        return false;
    };
    if !command_scan::shell_body_has_safe_static_shape(&run) {
        if !configuration.continue_on_error {
            *state.indeterminate |= configuration.condition != StaticBool::False;
            return true;
        }
        return false;
    }
    let pipefail_enforced = shell_pipefail_enforced(shell.as_deref());
    let reachable_run =
        command_scan::shell_body_before_static_failure(&run, failure_enforced, pipefail_enforced);
    let scanned = if failure_enforced {
        command_scan::scan_shell_for_typechecked_projects(&reachable_run, &cwd)
    } else {
        command_scan::scan_workflow_shell_for_typechecked_projects(&reachable_run, &cwd, false)
    };
    if !configuration.continue_on_error {
        for project in scanned {
            let project =
                resolve_gate_project_against_tracked(&project, configuration.context.tracked);
            if configuration
                .context
                .project_source_inputs
                .get(&project)
                .is_some_and(|source_inputs| {
                    source_inputs.iter().all(|input| {
                        matches!(
                            configuration.triggers.evaluate(input).0,
                            TriggerMatch::Matched | TriggerMatch::Always
                        )
                    })
                })
            {
                state.projects.insert(project);
            }
        }
    }
    let pipeline_failure = pipefail_enforced
        && command_scan::shell_body_has_static_pipeline_failure(&run, failure_enforced);
    let static_failure = pipeline_failure
        || command_scan::shell_body_has_static_failure_with_initial(&run, failure_enforced);
    if configuration.condition == StaticBool::True && static_failure {
        state.step_outcomes.record_with_conclusion(
            step,
            StaticValue::String("failure".to_string()),
            StaticValue::String(
                if configuration.continue_on_error {
                    "success"
                } else {
                    "failure"
                }
                .to_string(),
            ),
        );
        if !configuration.continue_on_error {
            *state.success = StaticBool::False;
            *state.failed = true;
        }
    } else if configuration.condition == StaticBool::True
        && command_scan::shell_body_is_statically_successful(&run)
    {
        state
            .step_outcomes
            .record(step, StaticValue::String("success".to_string()));
    }
    false
}
