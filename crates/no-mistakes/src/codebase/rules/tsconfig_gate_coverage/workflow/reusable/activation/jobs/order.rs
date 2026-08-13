use super::JobScanner;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    continues_after_failed_need, continues_after_indeterminate_need, continues_after_skipped_need,
    job_statically_disabled, job_statically_enabled, job_tolerates_failure, InputState,
    StaticValue,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::model::ActivationScan;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct JobOrderState {
    projects: BTreeSet<String>,
    completed: BTreeSet<String>,
    failed: BTreeSet<String>,
    indeterminate: BTreeSet<String>,
    runtime_skipped: BTreeSet<String>,
    known_executed: BTreeSet<String>,
    outputs: BTreeMap<String, BTreeMap<String, StaticValue>>,
}

struct DependencyState {
    failed: bool,
    skipped: bool,
    indeterminate: bool,
    continues: bool,
}

impl DependencyState {
    fn from_needs(
        job: &Value,
        needs: &[String],
        inputs: &[InputState],
        state: &JobOrderState,
    ) -> Self {
        let failed = needs.iter().any(|need| state.failed.contains(need));
        let skipped = needs
            .iter()
            .any(|need| state.runtime_skipped.contains(need));
        let indeterminate = needs.iter().any(|need| state.indeterminate.contains(need));
        let continues = inputs.iter().any(|inputs| {
            if failed {
                continues_after_failed_need(job, inputs)
            } else if skipped {
                continues_after_skipped_need(job, inputs)
            } else if indeterminate {
                continues_after_indeterminate_need(job, inputs)
            } else {
                false
            }
        });
        Self {
            failed,
            skipped,
            indeterminate,
            continues,
        }
    }

    fn unsuccessful(&self) -> bool {
        self.failed || self.skipped || self.indeterminate
    }

    fn blocks_indeterminately(&self) -> bool {
        self.indeterminate && !self.failed && !self.skipped && !self.continues
    }
}

struct JobDecision {
    skipped: bool,
    directly_enabled: bool,
    condition_indeterminate: bool,
}

impl JobDecision {
    fn new(
        job: &Value,
        job_id: &str,
        inputs: &[InputState],
        dependencies: &DependencyState,
        scanner: &JobScanner<'_, '_>,
    ) -> Self {
        let directly_disabled = !inputs.is_empty()
            && inputs
                .iter()
                .all(|inputs| job_statically_disabled(job, inputs));
        let directly_enabled = inputs
            .iter()
            .any(|inputs| job_statically_enabled(job, inputs));
        let condition_indeterminate = !directly_disabled
            && !inputs.is_empty()
            && !inputs
                .iter()
                .all(|inputs| job_statically_enabled(job, inputs));
        let zero_instances = scanner.job_states.has_zero_instances(job_id);
        let skipped = zero_instances
            || if dependencies.unsuccessful() {
                !dependencies.continues
            } else {
                directly_disabled
            };
        Self {
            skipped,
            directly_enabled,
            condition_indeterminate,
        }
    }
}

impl JobOrderState {
    fn record(
        &mut self,
        job_id: String,
        job: &Value,
        inputs: &[InputState],
        dependencies: &DependencyState,
        decision: &JobDecision,
        result: ActivationScan,
    ) {
        self.projects.extend(result.projects);
        if result.failed
            && inputs
                .iter()
                .any(|inputs| !job_tolerates_failure(job, inputs))
        {
            self.failed.insert(job_id.clone());
        }
        if result.indeterminate
            || dependencies.blocks_indeterminately()
            || decision.condition_indeterminate
        {
            self.indeterminate.insert(job_id.clone());
        }
        if decision.skipped {
            if !dependencies.blocks_indeterminately() {
                self.runtime_skipped.insert(job_id.clone());
            }
        } else if (dependencies.unsuccessful() && dependencies.continues)
            || (!dependencies.unsuccessful()
                && decision.directly_enabled
                && !decision.condition_indeterminate)
        {
            self.known_executed.insert(job_id.clone());
            self.outputs.insert(job_id.clone(), result.outputs);
        }
        self.completed.insert(job_id);
    }

    fn finish(self) -> ActivationScan {
        ActivationScan {
            projects: self.projects,
            outputs: BTreeMap::new(),
            job_outputs: self.outputs,
            failed: !self.failed.is_empty(),
            indeterminate: !self.indeterminate.is_empty(),
        }
    }
}

impl JobScanner<'_, '_> {
    pub(in super::super) fn scan(&mut self, jobs: &serde_yaml::Mapping) -> Option<ActivationScan> {
        let mut state = JobOrderState::default();
        while state.completed.len() < jobs.len() {
            let mut progressed = false;
            for (raw_job_id, job) in jobs {
                let job_id = super::super::super::super::normalized_job_id(raw_job_id)?;
                if state.completed.contains(&job_id) {
                    continue;
                }
                let needs = crate::codebase::workflow_topology::value_primitives::string_list(
                    job.get("needs"),
                )
                .into_iter()
                .map(|need| need.to_lowercase())
                .collect::<Vec<_>>();
                if !needs.iter().all(|need| state.completed.contains(need)) {
                    continue;
                }
                let inputs = self.job_states.inputs_with_results_for(
                    &job_id,
                    job,
                    &state.runtime_skipped,
                    &state.failed,
                    &state.known_executed,
                    &state.outputs,
                )?;
                let dependencies = DependencyState::from_needs(job, &needs, &inputs, &state);
                let decision = JobDecision::new(job, &job_id, &inputs, &dependencies, self);
                let result =
                    self.scan_job(&job_id, job, &inputs, decision.skipped, dependencies.failed)?;
                state.record(job_id, job, &inputs, &dependencies, &decision, result);
                progressed = true;
            }
            if !progressed {
                return None;
            }
        }
        Some(state.finish())
    }
}
