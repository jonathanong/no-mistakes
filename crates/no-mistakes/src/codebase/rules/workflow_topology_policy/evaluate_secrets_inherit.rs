use super::{job_finding, LocatedFinding, Options};
use crate::codebase::workflow_topology::model::{
    WorkflowCallSecretsBinding, WorkflowTopology, WorkflowTopologyEdge,
};
use crate::config::source_locations::ConfigPath;

pub(super) fn lint(topology: &WorkflowTopology, options: &Options) -> Vec<LocatedFinding> {
    if !options.forbid_secrets_inherit {
        return Vec::new();
    }
    topology
        .edges
        .iter()
        .filter_map(inherit_finding)
        .collect()
}

fn inherit_finding(edge: &WorkflowTopologyEdge) -> Option<LocatedFinding> {
    let WorkflowTopologyEdge::Calls(call) = edge else {
        return None;
    };
    if !matches!(call.bindings.secrets, WorkflowCallSecretsBinding::Inherit) {
        return None;
    }
    let (_, job) = call.from.rsplit_once('#')?;
    let mut finding = job_finding(
        &call.from,
        "forbidden secrets: inherit; map each secret explicitly on the reusable workflow call"
            .to_string(),
    );
    finding.path = ConfigPath::default().key("jobs").key(job).key("secrets");
    Some(finding)
}
