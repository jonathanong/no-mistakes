use super::JobScanner;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    job_statically_disabled, job_statically_enforcing, job_statically_not_enforcing,
    EnvironmentState, InputState, StaticBool,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::model::ActivationScan;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::steps::scan_job_steps;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::validation::{
    container_configuration_valid_for_inputs, strategy_configuration_valid_for_inputs,
    strategy_fail_fast_enabled_for_inputs,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::runtime::runner_os;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

use super::outputs::{merge_step_job_outputs, static_step_job_outputs};
impl JobScanner<'_, '_> {
    pub(super) fn scan_step_job(
        &self,
        job: &Value,
        inputs: &[InputState],
        skipped: bool,
        failed_need: bool,
    ) -> ActivationScan {
        if skipped {
            return ActivationScan {
                projects: BTreeSet::new(),
                outputs: BTreeMap::new(),
                job_outputs: BTreeMap::new(),
                failed: false,
                indeterminate: false,
            };
        }
        let mut projects = BTreeSet::new();
        let mut failed = false;
        let mut indeterminate = false;
        let mut outputs = None;
        let mut cancel_remaining_instances = false;
        for inputs in inputs {
            if cancel_remaining_instances {
                break;
            }
            let environment = EnvironmentState::from_workflow(
                self.workflow_runtime.workflow,
                &self.state.secrets,
                inputs,
            )
            .with_job(job, inputs)
            .with_runner_os(runner_os(job, inputs));
            match super::configuration::job_configuration_validity(job, inputs, &environment) {
                StaticBool::False => {
                    let enforcing = job_statically_enforcing(job, inputs, failed_need);
                    failed |= enforcing;
                    indeterminate |= !enforcing && !job_statically_disabled(job, inputs);
                    cancel_remaining_instances =
                        enforcing && strategy_fail_fast_enabled_for_inputs(job, inputs);
                    continue;
                }
                StaticBool::Invalid | StaticBool::Unknown | StaticBool::TruthyNonBoolean => {
                    indeterminate |= !job_statically_disabled(job, inputs);
                    continue;
                }
                StaticBool::True => {}
            }
            if !strategy_configuration_valid_for_inputs(job, inputs)
                || !container_configuration_valid_for_inputs(job, inputs, &environment)
            {
                let enforcing = job_statically_enforcing(job, inputs, failed_need);
                failed |= enforcing;
                indeterminate |= !enforcing && !job_statically_disabled(job, inputs);
                cancel_remaining_instances =
                    enforcing && strategy_fail_fast_enabled_for_inputs(job, inputs);
                continue;
            }
            if super::super::step_job_runner_supported(job, inputs) {
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
                    projects.extend(scan.projects.iter().cloned());
                }
                let enforcing = job_statically_enforcing(job, inputs, failed_need);
                if !scan.failed && !scan.indeterminate {
                    merge_step_job_outputs(
                        &mut outputs,
                        static_step_job_outputs(job, inputs, &environment),
                    );
                }
                let instance_failed = scan.failed && enforcing;
                failed |= instance_failed;
                indeterminate |= scan.indeterminate && enforcing;
                cancel_remaining_instances =
                    instance_failed && strategy_fail_fast_enabled_for_inputs(job, inputs);
            } else {
                // A statically unrunnable enabled job prevents downstream needs.
                let instance_failed = job_statically_enforcing(job, inputs, failed_need);
                failed |= instance_failed;
                cancel_remaining_instances =
                    instance_failed && strategy_fail_fast_enabled_for_inputs(job, inputs);
            }
        }
        ActivationScan {
            projects,
            outputs: outputs.unwrap_or_default(),
            job_outputs: BTreeMap::new(),
            failed,
            indeterminate,
        }
    }
}
