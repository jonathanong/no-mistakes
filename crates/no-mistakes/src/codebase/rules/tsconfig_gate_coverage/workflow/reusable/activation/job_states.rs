use std::collections::{BTreeMap, BTreeSet};

use serde_yaml::Value;

use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    inputs_with_matrix_values, inputs_with_needs_results, InputState, MatrixState,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::validation::{
    static_matrix_combinations, zero_instance_matrix, MatrixCombinations,
};

pub(super) struct JobStates {
    matrix_inputs: BTreeMap<String, Vec<InputState>>,
}

impl JobStates {
    pub(super) fn new(jobs: &serde_yaml::Mapping, inputs: &InputState) -> Option<Self> {
        let matrix_inputs = jobs
            .iter()
            .map(|(job_id, job)| {
                let job_id = super::super::super::normalized_job_id(job_id)?;
                let combinations = static_matrix_combinations(job)?;
                let inputs = match combinations {
                    MatrixCombinations::Static(combinations) => combinations
                        .iter()
                        .map(|values| {
                            inputs_with_matrix_values(inputs, values, MatrixState::Static)
                        })
                        .collect(),
                    MatrixCombinations::Dynamic(_) => vec![inputs_with_matrix_values(
                        inputs,
                        &BTreeMap::new(),
                        MatrixState::Dynamic,
                    )],
                };
                Some((job_id, inputs))
            })
            .collect::<Option<BTreeMap<_, Vec<_>>>>()?;
        Some(Self { matrix_inputs })
    }

    pub(super) fn has_zero_instances(&self, job: &Value) -> bool {
        zero_instance_matrix(job)
    }

    pub(super) fn inputs_for(&self, job_id: &str) -> Option<&[InputState]> {
        self.matrix_inputs.get(job_id).map(Vec::as_slice)
    }

    pub(super) fn inputs_with_results_for(
        &self,
        job_id: &str,
        job: &Value,
        runtime_skipped: &BTreeSet<String>,
        failed: &BTreeSet<String>,
        executed: &BTreeSet<String>,
    ) -> Option<Vec<InputState>> {
        Some(
            self.inputs_for(job_id)?
                .iter()
                .map(|inputs| {
                    inputs_with_needs_results(inputs, job, runtime_skipped, failed, executed)
                })
                .collect(),
        )
    }
}
