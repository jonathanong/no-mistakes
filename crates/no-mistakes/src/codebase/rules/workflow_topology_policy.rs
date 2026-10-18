use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod concurrency_compile;
mod concurrency_scope;
mod evaluate;
mod evaluate_collisions;
mod evaluate_concurrency;
mod evaluate_graph;
mod evaluate_steps;
#[cfg(test)]
mod prepare;
mod secret_inventory;

pub const RULE_ID: &str = "workflow-topology-policy";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) job_inventory: BTreeMap<String, Vec<String>>,
    pub(crate) unlocked_workflow_reasons: BTreeMap<String, String>,
    pub(crate) required_jobs: Vec<String>,
    pub(crate) forbidden_jobs: Vec<String>,
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
    pub(crate) secret_inventory: Option<secret_inventory::SecretInventory>,
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

pub(crate) fn check_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    _all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    inputs: &crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs,
    config_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
        let concurrency = concurrency_compile::compile(&opts.concurrency_policy)?;
        let inventory = opts
            .secret_inventory
            .as_ref()
            .map(|options| options.compile())
            .transpose()?;
        findings.extend(evaluate::lint(inputs.topology(), &opts, &concurrency));
        if let Some(inventory) = inventory {
            findings.extend(secret_inventory::lint(
                root,
                inputs,
                sources,
                &inventory,
                config_path,
            ));
        }
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(super) fn finding(message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: ".github/workflows".to_string(),
        line: 1,
        message,
        import: None,
        target: None,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod secret_inventory_tests;
