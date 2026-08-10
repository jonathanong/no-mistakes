use super::{reusable_call_target, scan_activation, step_job_runner_supported};
use crate::codebase::ci_graph::triggers::CompiledTriggers;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    callee_inputs, callee_secrets, inputs_with_matrix_values, statically_not_enforcing,
    statically_skipped_jobs, InputState,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::model::{
    ActivationMemo, ActivationState, ScanContext,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::steps::scan_job_steps;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::validation::{
    scan_job_shape_valid, static_matrix_combinations, validated_reusable_target,
    zero_instance_matrix,
};
use crate::codebase::workflow_topology::workflow_values;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct JobStates {
    matrix_inputs: BTreeMap<String, Vec<InputState>>,
    skipped: BTreeSet<String>,
}

impl JobStates {
    pub(super) fn new(jobs: &serde_yaml::Mapping, inputs: &InputState) -> Option<Self> {
        let zero_instance_jobs = zero_instance_job_ids(jobs)?;
        let matrix_inputs = jobs
            .iter()
            .map(|(job_id, job)| {
                let job_id = super::super::super::normalized_job_id(job_id)?;
                let combinations = static_matrix_combinations(job)?;
                let inputs = combinations
                    .iter()
                    .map(|values| inputs_with_matrix_values(inputs, values))
                    .collect();
                Some((job_id, inputs))
            })
            .collect::<Option<BTreeMap<_, Vec<_>>>>()?;
        let skipped = statically_skipped_jobs(jobs, &zero_instance_jobs, |job_id, _| {
            let job_id =
                super::super::super::normalized_job_id(job_id).expect("validated scalar job ID");
            matrix_inputs
                .get(&job_id)
                .cloned()
                .expect("matrix inputs were precomputed for every job")
        });
        Some(Self {
            matrix_inputs,
            skipped,
        })
    }

    fn is_skipped(&self, job_id: &str, job: &Value) -> bool {
        self.skipped.contains(job_id) || zero_instance_matrix(job)
    }

    fn inputs_for(&self, job_id: &str) -> Option<&[InputState]> {
        self.matrix_inputs.get(job_id).map(Vec::as_slice)
    }
}

pub(super) struct JobScanner<'a, 'workflow> {
    job_states: &'a JobStates,
    triggers: &'a CompiledTriggers,
    workflow_cwd: Option<String>,
    workflow_shell: Option<String>,
    state: &'a ActivationState,
    context: &'a ScanContext<'workflow>,
    memo: &'a mut ActivationMemo,
}

impl<'a, 'workflow> JobScanner<'a, 'workflow> {
    pub(super) fn new(
        job_states: &'a JobStates,
        triggers: &'a CompiledTriggers,
        workflow_cwd: Option<String>,
        workflow_shell: Option<String>,
        state: &'a ActivationState,
        context: &'a ScanContext<'workflow>,
        memo: &'a mut ActivationMemo,
    ) -> Self {
        Self {
            job_states,
            triggers,
            workflow_cwd,
            workflow_shell,
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
            if !skipped && !statically_not_enforcing(job, inputs) {
                projects.extend(callee_projects);
            }
        }
        Some(projects)
    }

    fn scan_step_job(&self, job: &Value, inputs: &[InputState], skipped: bool) -> BTreeSet<String> {
        if skipped || !step_job_runner_supported(job) {
            return BTreeSet::new();
        }
        let mut projects = BTreeSet::new();
        for inputs in inputs {
            if !statically_not_enforcing(job, inputs) {
                projects.extend(scan_job_steps(
                    job,
                    self.triggers,
                    inputs,
                    self.workflow_cwd.clone(),
                    self.workflow_shell.clone(),
                    self.context,
                ));
            }
        }
        projects
    }
}

fn zero_instance_job_ids(jobs: &serde_yaml::Mapping) -> Option<BTreeSet<String>> {
    jobs.iter()
        .filter(|(_, job)| zero_instance_matrix(job))
        .map(|(job_id, _)| super::super::super::normalized_job_id(job_id))
        .collect()
}
