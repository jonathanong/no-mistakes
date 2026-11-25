use super::{concurrency_compile, secret_inventory, RULE_ID};
use crate::codebase::rules::validate_options::{
    format_invalid_option, indexed_rule_applications, push_application_errors,
};
use crate::config::v2::NoMistakesConfig;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ForbiddenActivityTypes {
    pub(crate) types: Vec<String>,
    pub(crate) events: Option<Vec<String>>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ArtifactEdgeRule {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) name: String,
    #[serde(rename = "match")]
    pub(crate) match_kind: Option<String>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct StepOrderRule {
    pub(crate) job_id: String,
    pub(crate) steps: Vec<StepSelector>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct StepSelector {
    pub(crate) id: Option<String>,
    pub(crate) uses: Option<String>,
    pub(crate) name: Option<String>,
}

pub(crate) fn collect_config_errors(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    errors: &mut Vec<String>,
) {
    for (rule_index, rule) in indexed_rule_applications(config, RULE_ID) {
        let Some(value) = rule.options.get("requireExplicitWorkflowCallBindings") else {
            continue;
        };
        if !value.is_bool() {
            let mut app_errors = BTreeSet::new();
            app_errors.insert(format_invalid_option(
                RULE_ID,
                "requireExplicitWorkflowCallBindings",
                value,
                "a boolean",
            ));
            push_application_errors(root, config, rule_index, config_path, app_errors, errors);
        }
    }
}
