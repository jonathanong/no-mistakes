//! Reject directory enumeration of the live repository reachable from test roots.
mod assets;
pub(in crate::codebase::rules) mod config;
mod enumerators;
mod findings;
mod node_loaders;
mod owned;
pub(crate) use owned::Resolution;
mod provenance;
mod reachable;
mod registry;
mod requires;
mod subprocess;
use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use super::call_policy::{expand_roots, site_source_node, SharedRoots};
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::ts_source::module_facts::arguments::PathAnchor;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use crate::perf_trace::trace;
use anyhow::Result;
use config::Options;
use enumerators::{scan_of, Scan};
use provenance::{classify, Verdict};
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "test-no-live-repository-scan";

pub(crate) use config::graph_plan;
pub use config::vitest_roots_configured;

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
        let mut scope = trace("test-no-live-repository-scan.reachability", || {
            reachable::sources(graph, &roots, &root_files, graph_files, &setups)
        })?;
        let filter = super::path_filter::RulePathFilter::new(&root, config, application)?;
        for (site, expression) in trace("test-no-live-repository-scan.registry", || {
            registry::apply(
                graph,
                facts,
                &mut scope,
                &filter,
                &options.enumerators,
                resolution,
                graph_files,
            )
        }) {
            findings::unresolved_findings(&mut diagnostics, options.unresolved_root).push(
                findings::unresolved_dispatch(&root, application, site, &expression),
            );
        }
        let reachable = &scope.sources;
        let fixture_dirs = options.fixture_dirs();
        let analyzers = subprocess::Gate::new(&root, application, facts, &options, &fixture_dirs);
        let mut sites = requires::Sites::default();
        let owners = std::cell::OnceCell::new();
        trace("test-no-live-repository-scan.evaluate", || {
            for site in graph.resolved_call_sites() {
                if site.invocation != crate::codebase::dependencies::extract::InvocationKind::Call
                    || !reachable.contains(&site_source_node(site))
                    || !filter.is_match(&site.file)
                {
                    continue;
                }
                analyzers.note(&mut diagnostics, site);
                let loads =
                    node_loaders::findings(&root, application, site, facts, &options.enumerators);
                findings::unresolved_findings(&mut diagnostics, options.unresolved_root)
                    .extend(loads);
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
                let origin = ts.and_then(|ts| provenance::scan_origin(ts, site));
                let verdict = match scan {
                    Scan::Unresolved => {
                        Verdict::Unresolved("the receiver of this call is not resolved")
                    }
                    Scan::Resolved => classify(origin, &fixture_dirs),
                };
                let ownership = ((matches!(verdict, Verdict::Unresolved(_))
                    || origin.is_some_and(|origin| {
                        origin.anchor == PathAnchor::SharedTemp && origin.env.is_some()
                    }))
                    && scan == Scan::Resolved)
                    .then_some(origin)
                    .flatten()
                    .map(|origin| {
                        owners
                            .get_or_init(|| {
                                trace("test-no-live-repository-scan.ownership-index", || {
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
                            None => diagnostics.findings.push(findings::live(
                                &root,
                                application,
                                site,
                                why,
                            )),
                        }
                    }
                    Verdict::Unresolved(why) => {
                        let finding = match ownership {
                            Some(owned::Ownership::Parameter(cause)) => {
                                findings::parameter(&root, application, site, &cause)
                            }
                            Some(owned::Ownership::Environment(cause)) => {
                                findings::environment(&root, application, site, &cause)
                            }
                            _ => findings::unresolved(&root, application, site, why),
                        };
                        findings::unresolved_findings(&mut diagnostics, options.unresolved_root)
                            .push(finding);
                    }
                }
            }
        });
        for finding in sites.unresolved_requires(&root, application, facts, &options.enumerators) {
            findings::unresolved_findings(&mut diagnostics, options.unresolved_root).push(finding);
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
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
