use super::RuleFinding;
use crate::codebase::analysis_session::AnalysisSession;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

mod call_bindings_map;
mod call_bindings_yaml;
mod concurrency_compile;
mod concurrency_scope;
mod evaluate;
mod evaluate_activity_types;
mod evaluate_call_bindings;
mod evaluate_collisions;
mod evaluate_concurrency;
mod evaluate_forbidden_events;
mod evaluate_graph;
mod evaluate_secrets_inherit;
mod evaluate_steps;
mod execution;
mod locations;
mod malformed;
mod merge_availability;
mod options;
#[cfg(test)]
mod prepare;
mod secret_inventory;

pub use execution::diagnostics_prepared;
pub(crate) use options::{collect_config_errors, Options, StepSelector};

pub const RULE_ID: &str = "workflow-topology-policy";

pub(crate) fn check_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    session: Option<&AnalysisSession>,
    sources: &crate::codebase::ts_source::SourceStore,
    inputs: &crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs,
    config_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    diagnostics_prepared(root, config, session, sources, inputs, config_path, true)
        .map(|diagnostics| diagnostics.findings)
}

#[derive(Clone, Copy)]
pub(super) enum FindingOwner<'a> {
    Workflow(&'a str),
    Job(&'a str),
}

pub(super) struct LocatedFinding {
    finding: RuleFinding,
    path: crate::config::source_locations::ConfigPath,
    fallback_path: Option<crate::config::source_locations::ConfigPath>,
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
        fallback_path: None,
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
