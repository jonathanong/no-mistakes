use crate::codebase::workflow_topology::model::{
    JobMergeUncertainty, WorkflowMergeUncertainty, WorkflowTopology,
};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct MergeAvailability<'a>(BTreeMap<&'a str, &'a WorkflowMergeUncertainty>);

impl<'a> MergeAvailability<'a> {
    pub(super) fn new(topology: &'a WorkflowTopology) -> Self {
        Self(topology.diagnostics.iter().filter_map(|diagnostic| {
            Some((diagnostic.workflow_path.as_str(), diagnostic.merge_uncertainty.as_ref()?))
        }).collect())
    }

    pub(super) fn inventory_known(&self, workflow: &str) -> bool {
        self.0.get(workflow).is_none_or(|scope| !scope.job_inventory)
    }

    pub(super) fn job_membership_known(&self, job: &str) -> bool {
        self.inventory_known(workflow_of(job))
    }

    pub(super) fn steps_known(&self, job: &str, exists: bool) -> bool {
        (exists || self.job_membership_known(job)) && self.job(job).is_none_or(|scope| !scope.steps)
    }

    pub(super) fn concurrency_known(&self, owner: &str, exists: bool) -> bool {
        match owner.rsplit_once('#') {
            Some((file, _)) => {
                (exists || self.inventory_known(file))
                    && self.job(owner).is_none_or(|scope| !scope.concurrency)
            }
            None => self.0.get(owner).is_none_or(|scope| !scope.concurrency),
        }
    }

    pub(super) fn direct_edge_known(&self, from: &str, to: &str) -> bool {
        self.job_membership_known(from) && self.needs_known(to)
    }

    pub(super) fn needs_known(&self, job: &str) -> bool {
        self.job_membership_known(job) && self.job(job).is_none_or(|scope| !scope.needs)
    }

    pub(super) fn transitive_edges_known(&self, job: &str) -> bool {
        self.0.get(workflow_of(job)).is_none_or(|scope| {
            !scope.job_inventory && scope.jobs.values().all(|job| !job.needs)
        })
    }

    pub(super) fn callable_known(&self, workflow: &str) -> bool {
        self.0.get(workflow).is_none_or(|scope| !scope.callable)
    }

    pub(super) fn caller_known(&self, job: &str, exists: bool) -> bool {
        (exists || self.job_membership_known(job)) && self.job(job).is_none_or(|scope| !scope.uses)
    }

    pub(super) fn artifact_steps_known(&self, job: &str) -> bool {
        self.job(job).is_none_or(|scope| !scope.artifacts)
    }

    pub(super) fn artifacts_known(&self) -> bool {
        self.0.values().all(|scope| {
            !(scope.job_inventory || scope.callable || scope.call_contract)
                && scope.jobs.values().all(|job| !(job.uses || job.needs || job.artifacts))
        })
    }

    fn job(&self, id: &str) -> Option<&JobMergeUncertainty> {
        let (file, key) = id.rsplit_once('#')?;
        self.0.get(file)?.jobs.get(key)
    }
}

fn workflow_of(job: &str) -> &str {
    job.rsplit_once('#').map_or(job, |(file, _)| file)
}
