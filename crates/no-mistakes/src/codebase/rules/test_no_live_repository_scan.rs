//! Reject directory enumeration of the live repository reachable from test roots.
mod assets;
mod config;
mod enumerators;
mod findings;
mod node_loaders;
mod owned;
pub(crate) use owned::Resolution;
mod provenance;
mod reachable;
mod requires;

use super::analysis_policy::AnalysisDiagnostics;
use super::forbidden_calls::{call_argument_values, expand_roots, site_source_node, SharedRoots};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
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
    resolution: &Resolution<'_>,
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
        let setups = owned::setups::ConfiguredSetups::new(&root, vitest, &root_files);
        let scope = reachable::sources(graph, &roots, &root_files, graph_files, &setups)?;
        let reachable = &scope.sources;
        let filter = super::path_filter::RulePathFilter::new(&root, config, application)?;
        let fixture_dirs = options.fixture_dirs();
        let mut sites = requires::Sites::default();
        let owners = std::cell::OnceCell::new();
        for site in graph.resolved_call_sites() {
            if site.invocation != crate::codebase::dependencies::extract::InvocationKind::Call
                || !reachable.contains(&site_source_node(site))
                || !filter.is_match(&site.file)
            {
                continue;
            }
            let loads =
                node_loaders::findings(&root, application, site, facts, &options.enumerators);
            unresolved_findings(&mut diagnostics, options.unresolved_root).extend(loads);
            sites.visit(site, &options.enumerators);
            let Some(scan) = scan_of(site, &options.enumerators) else {
                continue;
            };
            let ts = facts.get_ts_facts(&site.file);
            // A mock that forwards its arguments unchanged is not a root of its own;
            // the callers of the mocked function are judged where they enumerate.
            if ts
                .and_then(owned::path_provenance)
                .and_then(|provenance| provenance.pass_through.get(&site.offset))
                .is_some_and(|spied| enumerators::spied_enumerator(spied, &options.enumerators))
            {
                continue;
            }
            let origin = ts
                .and_then(|ts| call_argument_values(ts, site.offset))
                .and_then(|values| values.path.as_ref());
            let verdict = match scan {
                Scan::Unresolved => {
                    Verdict::Unresolved("the receiver of this call is not resolved")
                }
                Scan::Resolved => classify(origin, &fixture_dirs),
            };
            let ownership = (matches!(verdict, Verdict::Unresolved(_)) && scan == Scan::Resolved)
                .then_some(origin)
                .flatten()
                .map(|origin| {
                    owners
                        .get_or_init(|| {
                            owned::Owners::new(
                                graph,
                                &scope,
                                facts,
                                graph_files,
                                resolution,
                                setups.clone(),
                                owned::Environment {
                                    root: &root,
                                    config,
                                    fixture_dirs: fixture_dirs
                                        .iter()
                                        .map(|dir| (*dir).to_owned())
                                        .collect(),
                                },
                            )
                        })
                        .ownership(origin, site, facts, &fixture_dirs)
                });
            let owned = matches!(ownership.as_ref(), Some(owned::Ownership::Yes));
            match verdict {
                Verdict::Allowed => {}
                _ if owned => {}
                Verdict::Live(why) => {
                    let asset = origin.and_then(|origin| {
                        assets::declared(origin, &root, &site.file, &options.runtime_assets)
                    });
                    match asset {
                        Some(asset) => diagnostics.advisories.push(findings::runtime_asset(
                            &root,
                            application,
                            site,
                            asset,
                        )),
                        None => {
                            diagnostics
                                .findings
                                .push(findings::live(&root, application, site, why))
                        }
                    }
                }
                Verdict::Unresolved(why) => {
                    let finding = match ownership {
                        Some(owned::Ownership::Parameter(cause)) => {
                            findings::parameter(&root, application, site, &cause)
                        }
                        _ => findings::unresolved(&root, application, site, why),
                    };
                    unresolved_findings(&mut diagnostics, options.unresolved_root).push(finding);
                }
            }
        }
        for finding in sites.unresolved_requires(&root, application, facts, &options.enumerators) {
            unresolved_findings(&mut diagnostics, options.unresolved_root).push(finding);
        }
        if let Some(owners) = owners.get() {
            owners.report_scan_stats();
        }
    }
    for list in [&mut diagnostics.findings, &mut diagnostics.advisories] {
        super::sort_findings(list);
        list.dedup();
    }
    Ok(diagnostics)
}

fn unresolved_findings(
    diagnostics: &mut AnalysisDiagnostics,
    policy: UnresolvedRoot,
) -> &mut Vec<super::RuleFinding> {
    match policy {
        UnresolvedRoot::Report => &mut diagnostics.findings,
        UnresolvedRoot::Advise => &mut diagnostics.advisories,
    }
}

#[cfg(test)]
mod tests;
