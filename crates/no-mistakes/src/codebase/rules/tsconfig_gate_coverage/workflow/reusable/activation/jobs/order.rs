use super::JobScanner;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    continues_after_failed_need, continues_after_skipped_need, job_statically_disabled,
    job_statically_enabled,
};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::model::ActivationScan;
use std::collections::BTreeSet;

impl JobScanner<'_, '_> {
    pub(in super::super) fn scan(&mut self, jobs: &serde_yaml::Mapping) -> Option<ActivationScan> {
        let mut projects = BTreeSet::new();
        let mut completed = BTreeSet::new();
        let mut failed = BTreeSet::new();
        let mut runtime_skipped = BTreeSet::new();
        let mut known_executed = BTreeSet::new();
        while completed.len() < jobs.len() {
            let mut progressed = false;
            for (raw_job_id, job) in jobs {
                let job_id = super::super::super::super::normalized_job_id(raw_job_id)?;
                if completed.contains(&job_id) {
                    continue;
                }
                let needs = crate::codebase::workflow_topology::value_primitives::string_list(
                    job.get("needs"),
                )
                .into_iter()
                .map(|need| need.to_lowercase())
                .collect::<Vec<_>>();
                if !needs.iter().all(|need| completed.contains(need)) {
                    continue;
                }
                let inputs = self.job_states.inputs_with_results_for(
                    &job_id,
                    job,
                    &runtime_skipped,
                    &failed,
                    &known_executed,
                )?;
                let failed_need = needs.iter().any(|need| failed.contains(need));
                let skipped_need = needs.iter().any(|need| runtime_skipped.contains(need));
                let continues = inputs.iter().any(|inputs| {
                    if failed_need {
                        continues_after_failed_need(job, inputs)
                    } else if skipped_need {
                        continues_after_skipped_need(job, inputs)
                    } else {
                        false
                    }
                });
                let unsuccessful_need = failed_need || skipped_need;
                let directly_disabled = !inputs.is_empty()
                    && inputs
                        .iter()
                        .all(|inputs| job_statically_disabled(job, inputs));
                let directly_enabled = inputs
                    .iter()
                    .any(|inputs| job_statically_enabled(job, inputs));
                let zero_instances = self.job_states.has_zero_instances(&job_id);
                let skipped = if unsuccessful_need {
                    zero_instances || !continues
                } else {
                    zero_instances || directly_disabled
                };
                let result = self.scan_job(&job_id, job, &inputs, skipped, failed_need)?;
                projects.extend(result.projects);
                if result.failed {
                    failed.insert(job_id.clone());
                }
                if skipped {
                    runtime_skipped.insert(job_id.clone());
                } else if (unsuccessful_need && continues)
                    || (!unsuccessful_need && directly_enabled)
                {
                    known_executed.insert(job_id.clone());
                }
                completed.insert(job_id);
                progressed = true;
            }
            if !progressed {
                return None;
            }
        }
        Some(ActivationScan {
            projects,
            failed: !failed.is_empty(),
        })
    }
}
