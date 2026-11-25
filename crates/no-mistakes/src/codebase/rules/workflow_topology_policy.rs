use super::RuleFinding;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::workflow_topology::model::DiagnosticCode;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

mod concurrency_compile;
mod concurrency_scope;
mod evaluate;
mod evaluate_activity_types;
mod evaluate_collisions;
mod evaluate_concurrency;
mod evaluate_forbidden_events;
mod evaluate_graph;
mod evaluate_secrets_inherit;
mod evaluate_steps;
mod locations;
mod malformed;
use malformed::report_malformed;
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
    pub(crate) forbidden_activity_types: Vec<ForbiddenActivityTypes>,
    pub(crate) forbidden_events: Vec<String>,
    pub(crate) forbid_secrets_inherit: bool,
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

pub(crate) fn check_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    session: Option<&AnalysisSession>,
    sources: &crate::codebase::ts_source::SourceStore,
    inputs: &crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs,
    config_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    let mut check_reusable = false;
    let mut locations = locations::Locations::default();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
        check_reusable |= opts.forbid_reusable_workflow_concurrency;
        let first_finding = findings.len();
        let concurrency = concurrency_compile::compile(&opts.concurrency_policy)?;
        let inventory = opts
            .secret_inventory
            .as_ref()
            .map(|options| options.compile())
            .transpose()?;
        let topology_findings = evaluate::lint(inputs.topology(), &opts, &concurrency);
        findings.extend(locations.locate(root, sources, topology_findings));
        if let Some(inventory) = inventory {
            let authored = config
                .rules
                .iter()
                .position(|entry| std::ptr::eq(entry, rule))
                .and_then(|index| {
                    config.config_provenance.origin(
                        &crate::config::source_locations::ConfigPath::default()
                            .key("rules")
                            .index(index),
                    )
                });
            findings.extend(secret_inventory::lint(
                root,
                inputs,
                sources,
                &inventory,
                authored.map(|(path, _)| path).or(config_path),
            ));
        }
        if let Some(message) = &rule.message {
            for finding in &mut findings[first_finding..] {
                finding.message.clone_from(message);
            }
        }
    }
    if check_reusable {
        for diagnostic in &inputs.topology().diagnostics {
            if diagnostic.code == DiagnosticCode::MalformedWorkflow {
                report_malformed(diagnostic, session)?;
            }
        }
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

#[derive(Clone, Copy)]
pub(super) enum FindingOwner<'a> {
    Workflow(&'a str),
    Job(&'a str),
}

pub(super) struct LocatedFinding {
    finding: RuleFinding,
    path: crate::config::source_locations::ConfigPath,
}

pub(super) fn finding(owner: FindingOwner<'_>, message: String) -> LocatedFinding {
    let (file, job) = match owner {
        FindingOwner::Workflow(file) => (file, None),
        FindingOwner::Job(id) => id
            .rsplit_once('#')
            .map_or((id, None), |(file, key)| (file, Some(key.to_owned()))),
    };
    LocatedFinding {
        finding: RuleFinding {
            causes: Vec::new(),
            rule: RULE_ID.to_string(),
            file: file.to_string(),
            line: 1,
            message,
            import: None,
            target: None,
        },
        path: job.map_or_else(
            crate::config::source_locations::ConfigPath::default,
            |job| {
                crate::config::source_locations::ConfigPath::default()
                    .key("jobs")
                    .key(job)
            },
        ),
    }
}

pub(super) fn workflow_finding(owner: &str, message: String) -> LocatedFinding {
    finding(FindingOwner::Workflow(owner), message)
}

pub(super) fn job_finding(owner: &str, message: String) -> LocatedFinding {
    finding(FindingOwner::Job(owner), message)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod secret_inventory_tests;
