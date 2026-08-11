use super::ScanContext;
use crate::codebase::ci_graph::triggers::{CompiledTriggers, TriggerMatch};
use crate::codebase::rules::tsconfig_gate_coverage::{
    application::resolve_gate_project_against_tracked, command_scan,
};
use serde_yaml::Value;
use std::collections::BTreeSet;

use super::super::{
    conditions::{
        statically_not_enforcing_with_environment, step_timeout_minutes_enforced, EnvironmentState,
        InputState,
    },
    effective_working_directory,
    runtime::{effective_shell, runs_on_can_default_to_windows, shell_failure_enforced},
};

pub(super) fn scan_job_steps(
    job: &Value,
    triggers: &CompiledTriggers,
    inputs: &InputState,
    environment: &EnvironmentState,
    workflow_cwd: Option<String>,
    workflow_shell: Option<String>,
    context: &ScanContext<'_>,
) -> BTreeSet<String> {
    let Some(steps) = job.get("steps").and_then(Value::as_sequence) else {
        return BTreeSet::new();
    };
    let job_cwd = effective_working_directory(job, workflow_cwd);
    let job_shell = effective_shell(job, workflow_shell);
    let implicit_shell_can_be_windows = runs_on_can_default_to_windows(job);
    let mut projects = BTreeSet::new();
    for step in steps {
        let environment = environment.with_step(step, inputs);
        if statically_not_enforcing_with_environment(step, inputs, &environment)
            || !step_timeout_minutes_enforced(step.get("timeout-minutes"), inputs)
        {
            continue;
        }
        let step_cwd = match step.get("working-directory").and_then(Value::as_str) {
            Some(raw) => command_scan::normalize_repo_relative(raw),
            None => job_cwd.clone(),
        };
        let Some(cwd) = step_cwd else {
            continue;
        };
        let Some(run) = step.get("run").and_then(Value::as_str) else {
            continue;
        };
        let shell = effective_shell(step, job_shell.clone());
        if shell.is_none() && implicit_shell_can_be_windows {
            continue;
        }
        let Some(failure_enforced) = shell_failure_enforced(shell.as_deref()) else {
            continue;
        };
        let scanned = if failure_enforced {
            command_scan::scan_shell_for_typechecked_projects(run, &cwd)
        } else {
            command_scan::scan_workflow_shell_for_typechecked_projects(run, &cwd, false)
        };
        for project in scanned {
            let project = resolve_gate_project_against_tracked(&project, context.tracked);
            if context
                .project_source_inputs
                .get(&project)
                .is_some_and(|source_inputs| {
                    source_inputs.iter().all(|input| {
                        matches!(
                            triggers.evaluate(input).0,
                            TriggerMatch::Matched | TriggerMatch::Always
                        )
                    })
                })
            {
                projects.insert(project);
            }
        }
    }
    projects
}
