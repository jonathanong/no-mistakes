use super::{workflow_finding, LocatedFinding, Options};
use crate::codebase::workflow_topology::model::WorkflowTopology;
use crate::config::source_locations::ConfigPath;
use rayon::prelude::*;
use std::collections::BTreeSet;

pub(super) fn lint(topology: &WorkflowTopology, options: &Options) -> Vec<LocatedFinding> {
    if options.forbidden_events.is_empty() {
        return Vec::new();
    }
    let forbidden: BTreeSet<&str> = options
        .forbidden_events
        .iter()
        .map(String::as_str)
        .collect();
    topology
        .workflows
        .par_iter()
        .flat_map_iter(|workflow| {
            let mut seen = BTreeSet::new();
            let mut findings = Vec::new();
            for trigger in &workflow.triggers {
                if !forbidden.contains(trigger.event.as_str()) || !seen.insert(trigger.event.as_str())
                {
                    continue;
                }
                let mut finding = workflow_finding(
                    &workflow.path,
                    format!(
                        "forbidden workflow event: {}; remove it from on to avoid unwanted workflow runs",
                        trigger.event
                    ),
                );
                finding.path = match trigger.config {
                    Some(_) => ConfigPath::default().key("on").key(&trigger.event),
                    None => ConfigPath::default().key("on"),
                };
                findings.push(finding);
            }
            findings
        })
        .collect()
}
