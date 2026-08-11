use std::collections::{BTreeMap, BTreeSet};

use serde_yaml::Value;

use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    inputs_with_matrix_values, statically_skipped_jobs, InputState, MatrixState,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::validation::{
    static_matrix_combinations, zero_instance_matrix, MatrixCombinations,
};

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

    pub(super) fn is_skipped(&self, job_id: &str, job: &Value) -> bool {
        self.skipped.contains(job_id) || zero_instance_matrix(job)
    }

    pub(super) fn inputs_for(&self, job_id: &str) -> Option<&[InputState]> {
        self.matrix_inputs.get(job_id).map(Vec::as_slice)
    }
}

fn zero_instance_job_ids(jobs: &serde_yaml::Mapping) -> Option<BTreeSet<String>> {
    jobs.iter()
        .filter(|(_, job)| zero_instance_matrix(job))
        .map(|(job_id, _)| super::super::super::normalized_job_id(job_id))
        .collect()
}
