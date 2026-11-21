use super::{list_conflict, within, Callee};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobSchedulerFactory {
    pub callee: String,
    pub template: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct JobSchedulerInput {
    pub(super) callee: String,
    pub(super) template: usize,
}

impl From<JobSchedulerInput> for JobSchedulerFactory {
    fn from(input: JobSchedulerInput) -> Self {
        Self {
            callee: input.callee,
            template: input.template,
        }
    }
}

impl From<&JobSchedulerFactory> for JobSchedulerInput {
    fn from(factory: &JobSchedulerFactory) -> Self {
        Self {
            callee: factory.callee.clone(),
            template: factory.template,
        }
    }
}

impl Callee for JobSchedulerFactory {
    fn callee(&self) -> &str {
        &self.callee
    }
}

pub(super) fn merge(
    into: &mut Vec<JobSchedulerFactory>,
    other: &[JobSchedulerFactory],
) -> Result<(), String> {
    if let Some(message) = list_conflict(into, other, same, "jobScheduler") {
        return Err(message);
    }
    into.extend(other.iter().cloned());
    Ok(())
}

pub(super) fn conflicts(items: &[JobSchedulerFactory]) -> Option<String> {
    within(items, same, "jobScheduler")
}

pub(super) fn dedup(items: &mut Vec<JobSchedulerFactory>) {
    items.sort_by(|left, right| left.callee.cmp(&right.callee));
    items.dedup_by(|left, right| left.callee == right.callee);
}

fn same(left: &JobSchedulerFactory, right: &JobSchedulerFactory) -> bool {
    left.template == right.template
}
