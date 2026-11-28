//! Direct single-enqueue calls inside open per-item loops and array callbacks.
pub(in crate::codebase::rules) mod modules;
pub(in crate::codebase::rules) mod options;
mod scan;
mod targets;

pub(crate) use modules::module_catalog;

use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use std::path::Path;

pub const RULE_ID: &str = "no-single-enqueue-in-loop";

/// Whether configured single targets, or options that fail to parse, need the call graph.
#[doc(hidden)]
pub fn demands_analysis(config: &NoMistakesConfig) -> bool {
    config.rule_applications(RULE_ID).iter().any(|application| {
        match application.try_rule_options::<options::Options>() {
            Ok(options) => options.has_single_targets(),
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

pub(crate) fn check_with_catalog(
    root: &Path,
    config: &NoMistakesConfig,
    graph: Option<&DepGraph>,
    facts: &crate::codebase::check_facts::CheckFactMap,
    catalog: Option<modules::ModuleCatalog<'_>>,
) -> Result<Vec<RuleFinding>> {
    let mut prepared = Vec::new();
    for application in config.rule_applications(RULE_ID) {
        let options = application.try_rule_options()?;
        let targets = options::validate(&options)?;
        if options.has_single_targets() {
            prepared.push((application, targets));
        }
    }
    if prepared.is_empty() {
        return Ok(Vec::new());
    }
    let Some(graph) = graph else {
        bail!("{RULE_ID} requires the canonical call graph");
    };
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let specifiers = prepared.iter().flat_map(|(_, targets)| {
        targets
            .single
            .iter()
            .chain(&targets.bulk)
            .map(|(module, _)| module.clone())
    });
    let resolved = modules::resolve_modules(&root, specifiers, facts, catalog)?;
    let mut findings = Vec::new();
    for (application, targets) in &prepared {
        findings.extend(scan::scan(
            &root,
            config,
            application,
            targets,
            graph,
            facts,
            &resolved,
        )?);
    }
    findings.sort();
    Ok(findings)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
