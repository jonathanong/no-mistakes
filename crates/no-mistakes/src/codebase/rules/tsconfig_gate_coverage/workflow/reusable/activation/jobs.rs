use super::{reusable_call_target, scan_activation, step_job_runner_supported};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    callee_inputs, callee_secrets, job_statically_disabled, job_statically_enforcing,
    job_statically_not_enforcing, EnvironmentState, InputState, StaticBool,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::model::ActivationScan;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::steps::scan_job_steps;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::validation::{
    container_configuration_valid_for_inputs, job_concurrency_valid_for_inputs,
    scan_job_shape_valid, strategy_configuration_valid_for_inputs, validated_reusable_target,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::runtime::runner_os;
use crate::codebase::workflow_topology::workflow_values;
use serde_yaml::Value;
use std::collections::BTreeSet;

mod configuration;
mod order;
mod scanner;
use configuration::job_configuration_validity;
pub(super) use scanner::{JobScanner, WorkflowRuntime};

impl<'a, 'workflow> JobScanner<'a, 'workflow> {
    fn scan_job(
        &mut self,
        job_id: &str,
        job: &Value,
        inputs: &[InputState],
        skipped: bool,
        failed_need: bool,
    ) -> Option<ActivationScan> {
        if !scan_job_shape_valid(job) {
            return None;
        }
        match reusable_call_target(job)? {
            Some(target) => {
                self.scan_reusable_job(job_id, target, job, inputs, skipped, failed_need)
            }
            None => Some(self.scan_step_job(job, inputs, skipped, failed_need)),
        }
    }

    fn scan_reusable_job(
        &mut self,
        job_id: &str,
        target: &str,
        job: &Value,
        inputs: &[InputState],
        skipped: bool,
        failed_need: bool,
    ) -> Option<ActivationScan> {
        let edge = workflow_values::call_edge(job_id, target, job);
        if !self.memo.register_target(validated_reusable_target(&edge)?)
            || self.state.active_paths.len() >= 10
        {
            return None;
        }
        if !edge.local {
            return Some(ActivationScan {
                projects: BTreeSet::new(),
                failed: false,
                indeterminate: false,
            });
        }
        let callee_path = edge.to.as_deref().unwrap_or_default();
        let callee = self.context.workflows.get(callee_path)?;
        if !callee.call_contract_shape_valid {
            return None;
        }
        let contract = callee.call_contract.as_ref()?;
        let callee_secrets = callee_secrets(contract, job, &self.state.secrets)?;
        let mut projects = BTreeSet::new();
        let mut failed = false;
        let mut indeterminate = false;
        let has_instances = !inputs.is_empty();
        let fallback_inputs;
        let inputs = if has_instances {
            inputs
        } else {
            fallback_inputs = self.state.inputs.clone();
            std::slice::from_ref(&fallback_inputs)
        };
        for inputs in inputs {
            if has_instances && !job_concurrency_valid_for_inputs(job.get("concurrency"), inputs) {
                failed |= !skipped && job_statically_enforcing(job, inputs, failed_need);
                continue;
            }
            let callee_inputs = callee_inputs(Some(contract), job, inputs)?;
            let callee_state = self.state.callee(callee_inputs, callee_secrets.clone());
            let callee_scan = scan_activation(
                callee_path,
                callee,
                self.triggers,
                &callee_state,
                self.context,
                self.memo,
            )?;
            if has_instances && !skipped {
                if !job_statically_not_enforcing(job, inputs) {
                    projects.extend(callee_scan.projects);
                }
                failed |= callee_scan.failed && job_statically_enforcing(job, inputs, failed_need);
                indeterminate |=
                    callee_scan.indeterminate && job_statically_enforcing(job, inputs, failed_need);
            }
        }
        Some(ActivationScan {
            projects,
            failed,
            indeterminate,
        })
    }

    fn scan_step_job(
        &self,
        job: &Value,
        inputs: &[InputState],
        skipped: bool,
        failed_need: bool,
    ) -> ActivationScan {
        if skipped {
            return ActivationScan {
                projects: BTreeSet::new(),
                failed: false,
                indeterminate: false,
            };
        }
        let mut projects = BTreeSet::new();
        let mut failed = false;
        let mut indeterminate = false;
        for inputs in inputs {
            let environment = EnvironmentState::from_workflow(
                self.workflow_runtime.workflow,
                &self.state.secrets,
                inputs,
            )
            .with_job(job, inputs)
            .with_runner_os(runner_os(job, inputs));
            match job_configuration_validity(job, inputs) {
                StaticBool::False => {
                    let enforcing = job_statically_enforcing(job, inputs, failed_need);
                    failed |= enforcing;
                    indeterminate |= !enforcing && !job_statically_disabled(job, inputs);
                    continue;
                }
                StaticBool::Unknown | StaticBool::TruthyNonBoolean => {
                    indeterminate |= !job_statically_disabled(job, inputs);
                    continue;
                }
                StaticBool::True => {}
            }
            if step_job_runner_supported(job, inputs)
                && strategy_configuration_valid_for_inputs(job, inputs)
                && container_configuration_valid_for_inputs(job, inputs, &environment)
            {
                let scan = scan_job_steps(
                    job,
                    self.triggers,
                    inputs,
                    &environment,
                    self.workflow_runtime.cwd.clone(),
                    self.workflow_runtime.shell.clone(),
                    self.context,
                );
                if !job_statically_not_enforcing(job, inputs) {
                    projects.extend(scan.projects);
                }
                failed |= scan.failed && job_statically_enforcing(job, inputs, failed_need);
                indeterminate |=
                    scan.indeterminate && job_statically_enforcing(job, inputs, failed_need);
            }
        }
        ActivationScan {
            projects,
            failed,
            indeterminate,
        }
    }
}
