use super::super::conditions::{InputState, SecretState};
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
pub(super) struct ActivationState {
    pub(super) inputs: InputState,
    pub(super) secrets: SecretState,
    pub(super) active_paths: BTreeSet<String>,
}

impl ActivationState {
    pub(super) fn direct(inputs: InputState) -> Self {
        Self {
            inputs,
            secrets: SecretState::direct(),
            active_paths: BTreeSet::new(),
        }
    }

    pub(super) fn callee(&self, inputs: InputState, secrets: SecretState) -> Self {
        Self {
            inputs,
            secrets,
            active_paths: self.active_paths.clone(),
        }
    }
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ActivationKey {
    pub(super) path: String,
    pub(super) state: ActivationState,
}

#[derive(Default)]
pub(super) struct ActivationMemo {
    entries: BTreeMap<ActivationKey, Option<BTreeSet<String>>>,
    computations: usize,
    targets: BTreeSet<ReusableTarget>,
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ReusableTarget {
    Local(String),
    Remote(String),
}

impl ActivationMemo {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn get(&self, key: &ActivationKey) -> Option<&Option<BTreeSet<String>>> {
        self.entries.get(key)
    }

    pub(super) fn record_computation(&mut self) {
        self.computations += 1;
    }

    pub(super) fn insert(&mut self, key: ActivationKey, result: Option<BTreeSet<String>>) {
        self.entries.insert(key, result);
    }

    pub(super) fn computations(&self) -> usize {
        self.computations
    }

    pub(super) fn register_target(&mut self, target: ReusableTarget) -> bool {
        self.targets.insert(target);
        self.targets.len() <= 50
    }
}
