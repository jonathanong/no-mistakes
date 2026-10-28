//! Reject directory enumeration of the live repository reachable from test roots.
mod config;
mod enumerators;
mod findings;
mod provenance;
mod reachable;
mod requires;

use super::analysis_policy::AnalysisDiagnostics;
use super::forbidden_calls::{call_argument_values, expand_roots, site_source_node, SharedRoots};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use config::{Options, UnresolvedRoot};
use enumerators::{scan_of, Scan};
use provenance::{classify, Verdict};
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "test-no-live-repository-scan";

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    config.rule_configured(RULE_ID).then(|| {
        let mut plan = GraphBuildPlan::imports_and_workspace();
        plan.calls = true;
        plan
    })
}

/// True when an application selects Vitest roots, so the catalog must be prepared.
pub fn vitest_roots_configured(config: &NoMistakesConfig) -> bool {
    config.rule_applications(RULE_ID).iter().any(|application| {
        application
            .options
            .get("roots")
            .and_then(serde_yaml::Value::as_sequence)
            .is_none_or(|roots| roots.iter().any(|root| root.get("vitest").is_some()))
    })
}

pub(crate) fn check_with_graph(
    root: &Path,
    config: &NoMistakesConfig,
    graph: &DepGraph,
    vitest: Option<&super::PreparedVitestProjectCatalog>,
    graph_files: &[PathBuf],
    facts: &dyn crate::codebase::dependencies::graph::TsFactLookup,
) -> Result<AnalysisDiagnostics> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut diagnostics = AnalysisDiagnostics::default();
    for application in config.rule_applications(RULE_ID) {
        let options: Options = application.try_rule_options()?;
        options.validate()?;
        let target_roots = super::target_roots(&root, config, application);
        let (roots, root_files) = expand_roots(SharedRoots {
            rule: RULE_ID,
            root: &root,
            roots: options.roots.as_ref(),
            exclude_roots: options.exclude_roots.as_ref(),
            graph,
            vitest,
            graph_files,
            target_roots: &target_roots,
        })?;
        let reachable = reachable::sources(graph, &roots, &root_files);
        let filter = super::path_filter::RulePathFilter::new(&root, config, application)?;
        let fixture_dirs = options.fixture_dirs();
        let mut sites = requires::Sites::default();
        for site in graph.resolved_call_sites() {
            if site.invocation != crate::codebase::dependencies::extract::InvocationKind::Call
                || !reachable.contains(&site_source_node(site))
                || !filter.is_match(&site.file)
            {
                continue;
            }
            sites.visit(site, &options.enumerators);
            let Some(scan) = scan_of(site, &options.enumerators) else {
                continue;
            };
            let origin = facts
                .get_ts_facts(&site.file)
                .and_then(|ts| call_argument_values(ts, site.offset))
                .and_then(|values| values.path.as_ref());
            let verdict = match scan {
                Scan::Unresolved => {
                    Verdict::Unresolved("the receiver of this call is not resolved")
                }
                Scan::Resolved => classify(origin, &fixture_dirs),
            };
            match verdict {
                Verdict::Allowed => {}
                Verdict::Live(why) => {
                    diagnostics
                        .findings
                        .push(findings::live(&root, application, site, why))
                }
                Verdict::Unresolved(why) => {
                    let finding = findings::unresolved(&root, application, site, why);
                    match options.unresolved_root {
                        UnresolvedRoot::Report => diagnostics.findings.push(finding),
                        UnresolvedRoot::Advise => diagnostics.advisories.push(finding),
                    }
                }
            }
        }
        for finding in sites.unresolved_requires(&root, application, facts, &options.enumerators) {
            match options.unresolved_root {
                UnresolvedRoot::Report => diagnostics.findings.push(finding),
                UnresolvedRoot::Advise => diagnostics.advisories.push(finding),
            }
        }
    }
    for list in [&mut diagnostics.findings, &mut diagnostics.advisories] {
        super::sort_findings(list);
        list.dedup();
    }
    Ok(diagnostics)
}

#[cfg(test)]
mod tests;
