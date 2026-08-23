use super::super::job_states::JobStates;
use crate::codebase::ci_graph::triggers::CompiledTriggers;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::reusable::model::{
    ActivationMemo, ActivationState, ScanContext,
};
use serde_yaml::Value;

pub(crate) struct WorkflowRuntime<'workflow> {
    pub(crate) cwd: Option<String>,
    pub(crate) shell: Option<String>,
    pub(crate) workflow: &'workflow Value,
}

pub(crate) struct JobScanner<'a, 'workflow> {
    pub(crate) job_states: &'a JobStates,
    pub(crate) triggers: &'a CompiledTriggers,
    pub(crate) workflow_runtime: WorkflowRuntime<'workflow>,
    pub(crate) state: &'a ActivationState,
    pub(crate) context: &'a ScanContext<'workflow>,
    pub(crate) memo: &'a mut ActivationMemo,
}

impl<'a, 'workflow> JobScanner<'a, 'workflow> {
    pub(crate) fn new(
        job_states: &'a JobStates,
        triggers: &'a CompiledTriggers,
        workflow_runtime: WorkflowRuntime<'workflow>,
        state: &'a ActivationState,
        context: &'a ScanContext<'workflow>,
        memo: &'a mut ActivationMemo,
    ) -> Self {
        Self {
            job_states,
            triggers,
            workflow_runtime,
            state,
            context,
            memo,
        }
    }
}
