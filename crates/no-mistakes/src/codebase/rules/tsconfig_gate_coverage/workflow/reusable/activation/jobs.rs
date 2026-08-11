use super::{reusable_call_target, scan_activation, step_job_runner_supported};
use crate::codebase::ci_graph::triggers::CompiledTriggers;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    callee_inputs, callee_secrets, job_timeout_minutes_enforced, statically_not_enforcing,
    EnvironmentState, InputState,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::model::{
    ActivationMemo, ActivationState, ScanContext,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::steps::scan_job_steps;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::validation::{
    container_configuration_valid_for_inputs, environment_configuration_valid_for_inputs,
    scan_job_shape_valid, strategy_configuration_valid_for_inputs, validated_reusable_target,
};
use crate::codebase::workflow_topology::workflow_values;
use serde_yaml::Value;
use std::collections::BTreeSet;

use super::job_states::JobStates;

pub(super) struct WorkflowRuntime<'workflow> {
    pub(super) cwd: Option<String>,
    pub(super) shell: Option<String>,
    pub(super) workflow: &'workflow Value,
}

pub(super) struct JobScanner<'a, 'workflow> {
    job_states: &'a JobStates,
    triggers: &'a CompiledTriggers,
    workflow_runtime: WorkflowRuntime<'workflow>,
    state: &'a ActivationState,
    context: &'a ScanContext<'workflow>,
    memo: &'a mut ActivationMemo,
}

impl<'a, 'workflow> JobScanner<'a, 'workflow> {
    pub(super) fn new(
        job_states: &'a JobStates,
        triggers: &'a CompiledTriggers,
        workflow_runtime: WorkflowRuntime<'workflow>,
        state: &'a ActivationState,
        context: &'a ScanContext<'workflow>,
        memo: &'a mut ActivationMemo,
    ) -> Self {
        Self {
            job_states,
            triggers,
            workflow_runtime,
            state,
            context,
            memo,
        }
    }

    pub(super) fn scan(&mut self, jobs: &serde_yaml::Mapping) -> Option<BTreeSet<String>> {
        let mut projects = BTreeSet::new();
        for (job_id, job) in jobs {
            let job_id = super::super::super::normalized_job_id(job_id)?;
            projects.extend(self.scan_job(&job_id, job)?);
        }
        Some(projects)
    }

    fn scan_job(&mut self, job_id: &str, job: &Value) -> Option<BTreeSet<String>> {
        if !scan_job_shape_valid(job) {
            return None;
        }
        let inputs = self.job_states.inputs_for(job_id)?;
        let skipped = self.job_states.is_skipped(job_id, job);
        match reusable_call_target(job)? {
            Some(target) => self.scan_reusable_job(job_id, target, job, inputs, skipped),
            None => Some(self.scan_step_job(job, inputs, skipped)),
        }
    }

    fn scan_reusable_job(
        &mut self,
        job_id: &str,
        target: &str,
        job: &Value,
        inputs: &[InputState],
        skipped: bool,
    ) -> Option<BTreeSet<String>> {
        let edge = workflow_values::call_edge(job_id, target, job);
        if !self.memo.register_target(validated_reusable_target(&edge)?)
            || self.state.active_paths.len() >= 10
        {
            return None;
        }
        if !edge.local {
            return Some(BTreeSet::new());
        }
        let callee_path = edge.to.as_deref().unwrap_or_default();
        let callee = self.context.workflows.get(callee_path)?;
        if !callee.call_contract_shape_valid {
            return None;
        }
        let contract = callee.call_contract.as_ref()?;
        let callee_secrets = callee_secrets(contract, job, &self.state.secrets)?;
        let mut projects = BTreeSet::new();
        let has_instances = !inputs.is_empty();
        let fallback_inputs;
        let inputs = if has_instances {
            inputs
        } else {
            fallback_inputs = self.state.inputs.clone();
            std::slice::from_ref(&fallback_inputs)
        };
        for inputs in inputs {
            let callee_inputs = callee_inputs(Some(contract), job, inputs)?;
            let callee_state = self.state.callee(callee_inputs, callee_secrets.clone());
            let callee_projects = scan_activation(
                callee_path,
                callee,
                self.triggers,
                &callee_state,
                self.context,
                self.memo,
            )?;
            if has_instances && !skipped && !statically_not_enforcing(job, inputs) {
                projects.extend(callee_projects);
            }
        }
        Some(projects)
    }

    fn scan_step_job(&self, job: &Value, inputs: &[InputState], skipped: bool) -> BTreeSet<String> {
        if skipped {
            return BTreeSet::new();
        }
        let mut projects = BTreeSet::new();
        for inputs in inputs {
            let environment = EnvironmentState::from_workflow(
                self.workflow_runtime.workflow,
                &self.state.secrets,
                inputs,
            )
            .with_job(job, inputs);
            if step_job_runner_supported(job, inputs)
                && strategy_configuration_valid_for_inputs(job, inputs)
                && job_timeout_minutes_enforced(job.get("timeout-minutes"), inputs)
                && environment_configuration_valid_for_inputs(job, inputs)
                && !statically_not_enforcing(job, inputs)
                && container_configuration_valid_for_inputs(job, inputs, &environment)
            {
                projects.extend(scan_job_steps(
                    job,
                    self.triggers,
                    inputs,
                    &environment,
                    self.workflow_runtime.cwd.clone(),
                    self.workflow_runtime.shell.clone(),
                    self.context,
                ));
            }
        }
        projects
    }
}
