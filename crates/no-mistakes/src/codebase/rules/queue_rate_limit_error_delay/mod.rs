//! Require a positive delay when throwing or constructing a configured rate-limit error.
mod format;
mod judge;
mod matching;
mod options;
mod scan;

use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::rules::no_single_enqueue_in_loop::modules::{resolve_modules, ModuleCatalog};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use std::path::Path;

pub const RULE_ID: &str = "queue-rate-limit-error-delay";

pub(crate) use options::collect_application_errors;

#[doc(hidden)]
pub fn demands_analysis(config: &NoMistakesConfig) -> bool {
    !config.rule_applications(RULE_ID).is_empty()
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
    catalog: Option<ModuleCatalog<'_>>,
) -> Result<AnalysisDiagnostics> {
    let mut prepared = Vec::new();
    for application in config.rule_applications(RULE_ID) {
        let policy = config.effective_rule_analysis(application)?.unanalyzable;
        let opts: options::Options = application.try_rule_options()?;
        let targets = options::validate(&opts)?;
        prepared.push((application, policy, targets));
    }
    if prepared.is_empty() {
        return Ok(AnalysisDiagnostics::default());
    }
    let Some(graph) = graph else {
        bail!("{RULE_ID} requires the canonical call graph");
    };
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let specifiers = prepared
        .iter()
        .flat_map(|(_, _, targets)| targets.iter().map(|target| target.module.clone()));
    let resolved = resolve_modules(&root, specifiers, facts, catalog)?;
    let mut diagnostics = AnalysisDiagnostics::default();
    for (application, policy, targets) in &prepared {
        diagnostics.append(scan::scan(&scan::ScanInput {
            root: &root,
            config,
            application,
            policy: *policy,
            targets,
            graph,
            facts,
            resolved: &resolved,
        })?);
    }
    crate::codebase::rules::sort_findings(&mut diagnostics.findings);
    crate::codebase::rules::sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
