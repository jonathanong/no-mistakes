use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod evaluate;
mod evaluate_graph;
mod evaluate_steps;

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

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    _all_files: &[PathBuf],
    _sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let topology =
            crate::codebase::workflow_topology::load_workflow_topology(root, &config.ci, &[]);
        findings.extend(evaluate::lint(&topology, &opts));
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
mod coverage_step_tests;
#[cfg(test)]
mod coverage_support;
#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod tests;
