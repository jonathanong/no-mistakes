use super::super::conditions::{InputState, SecretState, StaticValue};
use crate::codebase::rules::tsconfig_gate_coverage::ProjectSourceInputs;
use crate::codebase::workflow_topology::model::WorkflowCallContract;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

/// A single direct workflow event may evaluate at most this many distinct
/// path-sensitive reusable-workflow activation states. This bounds layered
/// call DAGs whose resolved inputs otherwise create exponentially many memo
/// keys, while leaving ordinary reusable-workflow graphs room to share work.
pub(crate) const MAX_ACTIVATION_COMPUTATIONS: usize = 1024;

pub(super) struct WorkflowDocument<'a> {
    pub(super) value: &'a Value,
    pub(super) call_contract: Option<WorkflowCallContract>,
    pub(super) call_contract_shape_valid: bool,
}

pub(super) struct ScanContext<'a> {
    pub(super) workflows: BTreeMap<String, WorkflowDocument<'a>>,
    pub(super) tracked: &'a BTreeSet<String>,
    pub(super) project_source_inputs: &'a ProjectSourceInputs,
    pub(super) local_actions: &'a BTreeSet<String>,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum GithubEventAction {
    Missing,
    Known(String),
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum GithubRef {
    Exact(String),
    UnknownExcluding(BTreeSet<String>),
    PullRequestMerge,
    Unknown,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct GithubEventContext {
    pub(crate) name: String,
    pub(crate) action: GithubEventAction,
    pub(crate) reference: GithubRef,
    pub(crate) base_reference: GithubRef,
}

impl GithubEventContext {
    pub(crate) fn with_action_and_refs(
        name: &str,
        action: &str,
        reference: GithubRef,
        base_reference: GithubRef,
    ) -> Self {
        Self {
            name: name.to_string(),
            action: GithubEventAction::Known(action.to_string()),
            reference,
            base_reference,
        }
    }

    pub(crate) fn with_ref(name: &str, reference: GithubRef) -> Self {
        Self {
            name: name.to_string(),
            action: GithubEventAction::Missing,
            reference,
            base_reference: GithubRef::Unknown,
        }
    }
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

#[derive(Clone)]
pub(super) struct ActivationScan {
    pub(super) projects: BTreeSet<String>,
    pub(super) outputs: BTreeMap<String, StaticValue>,
    pub(super) failed: bool,
    pub(super) indeterminate: bool,
}

#[derive(Default)]
pub(super) struct ActivationMemo {
    entries: BTreeMap<ActivationKey, Option<ActivationScan>>,
    computations: usize,
    exhausted: bool,
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

    pub(super) fn get(&self, key: &ActivationKey) -> Option<&Option<ActivationScan>> {
        self.entries.get(key)
    }

    pub(super) fn try_record_computation(&mut self) -> bool {
        if self.computations >= MAX_ACTIVATION_COMPUTATIONS {
            self.exhausted = true;
            return false;
        }
        self.computations += 1;
        true
    }

    pub(super) fn insert(&mut self, key: ActivationKey, result: Option<ActivationScan>) {
        self.entries.insert(key, result);
    }

    pub(super) fn computations(&self) -> usize {
        self.computations
    }

    pub(super) fn exhausted(&self) -> bool {
        self.exhausted
    }

    pub(super) fn register_target(&mut self, target: ReusableTarget) -> bool {
        self.targets.insert(target);
        self.targets.len() <= 50
    }
}
