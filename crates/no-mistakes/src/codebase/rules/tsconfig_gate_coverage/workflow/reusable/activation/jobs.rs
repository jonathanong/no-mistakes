use super::{reusable_call_target, scan_activation};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    callee_inputs, callee_secrets, job_statically_enforcing, job_statically_not_enforcing,
    InputState, StaticValue,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::model::ActivationScan;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::validation::{
    job_concurrency_valid_for_inputs, scan_job_shape_valid, strategy_fail_fast_enabled_for_inputs,
    validated_reusable_target,
};
use crate::codebase::workflow_topology::workflow_values;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

mod configuration;
mod order;
mod outputs;
mod scanner;
mod step;
use outputs::merge_reusable_outputs;
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
                outputs: BTreeMap::new(),
                job_outputs: BTreeMap::new(),
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
        let mut projects = BTreeSet::new();
        let mut outputs: Option<BTreeMap<String, StaticValue>> = None;
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
        let mut cancel_remaining_instances = false;
        for inputs in inputs {
            if cancel_remaining_instances {
                break;
            }
            if has_instances && !job_concurrency_valid_for_inputs(job.get("concurrency"), inputs) {
                let instance_failed =
                    !skipped && job_statically_enforcing(job, inputs, failed_need);
                failed |= instance_failed;
                cancel_remaining_instances =
                    instance_failed && strategy_fail_fast_enabled_for_inputs(job, inputs);
                continue;
            }
            let callee_secrets = callee_secrets(contract, job, &self.state.secrets, inputs)?;
            let callee_inputs = callee_inputs(Some(contract), job, inputs)?;
            let callee_state = self.state.callee(callee_inputs, callee_secrets);
            let callee_scan = scan_activation(
                callee_path,
                callee,
                self.triggers,
                &callee_state,
                self.context,
                self.memo,
            );
            let callee_scan = callee_scan?;
            if has_instances && !skipped {
                merge_reusable_outputs(&mut outputs, &callee_scan);
                if !job_statically_not_enforcing(job, inputs) {
                    projects.extend(callee_scan.projects);
                }
                let enforcing = job_statically_enforcing(job, inputs, failed_need);
                let instance_failed = callee_scan.failed && enforcing;
                failed |= instance_failed;
                indeterminate |= callee_scan.indeterminate && enforcing;
                cancel_remaining_instances =
                    instance_failed && strategy_fail_fast_enabled_for_inputs(job, inputs);
            }
        }
        Some(ActivationScan {
            projects,
            outputs: outputs.unwrap_or_default(),
            job_outputs: BTreeMap::new(),
            failed,
            indeterminate,
        })
    }
}
