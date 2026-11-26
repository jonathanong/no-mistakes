use super::{
    concurrency_compile, secret_inventory, ArtifactEdgeRule, ForbiddenActivityTypes, StepOrderRule,
};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) job_inventory: BTreeMap<String, Vec<String>>,
    pub(crate) unlocked_workflow_reasons: BTreeMap<String, String>,
    pub(crate) required_jobs: Vec<String>,
    pub(crate) forbidden_jobs: Vec<String>,
    pub(crate) forbidden_activity_types: Vec<ForbiddenActivityTypes>,
    pub(crate) forbidden_events: Vec<String>,
    pub(crate) forbid_secrets_inherit: bool,
    pub(crate) require_explicit_workflow_call_bindings: bool,
    pub(crate) required_direct_edges: Vec<[String; 2]>,
    pub(crate) forbidden_direct_edges: Vec<[String; 2]>,
    pub(crate) required_transitive_edges: Vec<[String; 2]>,
    pub(crate) forbidden_transitive_edges: Vec<[String; 2]>,
    pub(crate) required_artifact_edges: Vec<ArtifactEdgeRule>,
    pub(crate) exact_fan_ins: BTreeMap<String, Vec<String>>,
    pub(crate) exact_caller_jobs: BTreeMap<String, Vec<String>>,
    pub(crate) step_orders: Vec<StepOrderRule>,
    pub(crate) concurrency_policy: BTreeMap<String, concurrency_compile::ConcurrencyIntent>,
    pub(crate) forbid_concurrency_group_collisions: bool,
    pub(crate) forbid_reusable_workflow_concurrency: bool,
    pub(crate) secret_inventory: Option<secret_inventory::SecretInventory>,
}
