use super::{workflow_finding, LocatedFinding, Options};
use crate::codebase::workflow_topology::{model::WorkflowTopology, value_primitives::OrderedJson};
use crate::config::source_locations::ConfigPath;
use rayon::prelude::*;
use std::collections::BTreeSet;

pub(super) fn lint(topology: &WorkflowTopology, options: &Options) -> Vec<LocatedFinding> {
    if options.forbidden_activity_types.is_empty() {
        return Vec::new();
    }
    topology
        .workflows
        .par_iter()
        .flat_map_iter(|workflow| {
            let mut findings = Vec::new();
            for trigger in &workflow.triggers {
                let forbidden: BTreeSet<&str> = options
                    .forbidden_activity_types
                    .iter()
                    .filter(|policy| {
                        policy.events.as_ref().is_none_or(|events| events.contains(&trigger.event))
                    })
                    .flat_map(|policy| policy.types.iter().map(String::as_str))
                    .collect();
                let types = trigger.config.as_ref().and_then(|config| config.get("types"));
                let declared: BTreeSet<&str> = match types {
                    Some(OrderedJson::String(value)) => [value.as_str()].into_iter().collect(),
                    Some(OrderedJson::Array(values)) => {
                        values.iter().filter_map(OrderedJson::as_str).collect()
                    }
                    _ => BTreeSet::new(),
                };
                for activity in declared.intersection(&forbidden) {
                    let mut finding = workflow_finding(
                        &workflow.path,
                        format!(
                            "forbidden workflow activity type: {}: {activity}; remove it from on.{}.types to avoid unwanted workflow runs",
                            trigger.event, trigger.event
                        ),
                    );
                    finding.path = ConfigPath::default().key("on").key(&trigger.event).key("types");
                    findings.push(finding);
                }
            }
            findings
        })
        .collect()
}
