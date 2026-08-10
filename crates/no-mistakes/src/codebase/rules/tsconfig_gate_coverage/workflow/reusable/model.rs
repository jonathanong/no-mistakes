use super::super::conditions::InputState;
use crate::codebase::rules::tsconfig_gate_coverage::ProjectSourceInputs;
use crate::codebase::workflow_topology::model::WorkflowCallContract;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct WorkflowDocument<'a> {
    pub(super) value: &'a Value,
    pub(super) call_contract: Option<WorkflowCallContract>,
    pub(super) call_contract_shape_valid: bool,
}

pub(super) struct ScanContext<'a> {
    pub(super) workflows: BTreeMap<String, WorkflowDocument<'a>>,
    pub(super) tracked: &'a BTreeSet<String>,
    pub(super) project_source_inputs: &'a ProjectSourceInputs,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ActivationKey {
    pub(super) path: String,
    pub(super) inputs: InputState,
    pub(super) active_paths: BTreeSet<String>,
}

pub(super) type ActivationMemo = BTreeMap<ActivationKey, Option<BTreeSet<String>>>;
