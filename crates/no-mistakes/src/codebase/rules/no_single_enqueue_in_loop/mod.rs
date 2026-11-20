//! Direct single-enqueue calls inside open per-item loops and array callbacks.
mod options;
mod scan;

use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use std::path::Path;

pub const RULE_ID: &str = "no-single-enqueue-in-loop";

/// Whether configured single targets, or options that fail to parse, need the call graph.
#[doc(hidden)]
pub fn demands_analysis(config: &NoMistakesConfig) -> bool {
    config.rule_applications(RULE_ID).iter().any(|application| {
        match application.try_rule_options::<options::Options>() {
            Ok(options) => !options.single.is_empty(),
            Err(_) => true,
        }
    })
}

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    demands_analysis(config).then(|| GraphBuildPlan {
        calls: true,
        ..GraphBuildPlan::default()
    })
}

pub(crate) fn check(
    root: &Path,
    config: &NoMistakesConfig,
    graph: Option<&DepGraph>,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let mut prepared = Vec::new();
    for application in config.rule_applications(RULE_ID) {
        let options = application.try_rule_options()?;
        let targets = options::validate(&options)?;
        if targets.single.is_empty() {
            continue;
        }
        prepared.push((application, targets));
    }
    if prepared.is_empty() {
        return Ok(Vec::new());
    }
    let Some(graph) = graph else {
        bail!("{RULE_ID} requires the canonical call graph");
    };
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut findings = Vec::new();
    for (application, targets) in &prepared {
        findings.extend(scan::scan(
            &root,
            config,
            application,
            targets,
            graph,
            facts,
        )?);
    }
    findings.sort();
    Ok(findings)
}

#[cfg(test)]
mod tests;
