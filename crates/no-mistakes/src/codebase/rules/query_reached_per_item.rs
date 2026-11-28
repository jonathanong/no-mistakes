//! Per-item policies project parser-owned positions and canonical call traces.
mod prerequisites;
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
mod allow;
mod candidate_policy;
mod candidates;
pub(in crate::codebase::rules) mod config;
mod languages;
mod traces;
pub(crate) use config::graph_plan;
use config::{exempt, Options, Sinks};
use traces::{prefer_nontransaction, source_node, Reach};
pub const RULE_ID: &str = "query-reached-per-item";
const SINK_SITES_METRIC: &str = "query_reached_per_item.sink_sites";
pub(crate) fn check_with_graph_and_session(
    root: &Path,
    config: &NoMistakesConfig,
    graph: &DepGraph,
    facts: &CheckFactMap,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let root = root.as_path();
    let mut findings = Vec::new();
    for application in config.rule_applications(RULE_ID) {
        let options: Options = application.try_rule_options()?;
        config::validate(&options, config)?;
        let filter = allow::filter(root, config, application, sources)?;
        let audit = super::allow_audit::AllowAudit::new_files(
            root,
            config,
            application,
            sources,
            "allow",
            options.allow.len(),
        );
        prerequisites::require_sources(graph, facts, &filter, session)?;
        let scoped_sites = candidates::select(graph, facts, &filter);
        let observer = crate::diagnostics::current();
        let effects = if scoped_sites.is_empty() {
            if let Some(observer) = &observer {
                observer.increment(SINK_SITES_METRIC, 0);
            }
            &[]
        } else {
            options.effects.as_slice()
        };
        for kind in effects {
            let effect = &config.effects[kind];
            let configured_sinks = Sinks::new(effect, graph);
            let mut sinks: BTreeMap<NodeId, Vec<(&ResolvedCallSite, bool)>> = BTreeMap::new();
            let mut excluded = Vec::new();
            let resolved_sites = graph.resolved_call_sites();
            if let Some(observer) = &observer {
                observer.increment(SINK_SITES_METRIC, resolved_sites.len() as u64);
            }
            for site in resolved_sites {
                if site
                    .caller
                    .as_deref()
                    .is_some_and(|caller| exempt(effect, caller))
                {
                    excluded.push(source_node(site));
                }
                if exempt(effect, &site.source_callee) {
                    excluded.extend(site.target_nodes().cloned());
                } else if let Some(transaction) = configured_sinks.match_site(site) {
                    sinks
                        .entry(source_node(site))
                        .or_default()
                        .push((site, transaction));
                }
            }
            excluded.sort();
            excluded.dedup();
            let policy = candidate_policy::CandidatePolicy {
                sinks: &configured_sinks,
                options: &options,
                effect,
                excluded: &excluded,
            };
            let candidates: Vec<_> = scoped_sites
                .iter()
                .copied()
                .filter(|(site, position)| policy.includes(site, position))
                .collect();
            let mut roots = candidates
                .iter()
                .flat_map(|(site, _)| site.target_nodes().cloned())
                .collect::<Vec<_>>();
            roots.sort();
            roots.dedup();
            roots.retain(|node| excluded.binary_search(node).is_err());
            prerequisites::require_reachable(
                graph,
                candidates.iter().map(|(site, _)| *site),
                &excluded,
                session,
            )?;
            let reachable = traces::reachable(graph, &roots, &sinks, &excluded);
            for (site, position) in candidates {
                let direct = configured_sinks.match_site(site).map(|transaction| Reach {
                    path: vec![site.source_callee.clone()],
                    transaction,
                });
                let transitive = site
                    .target_nodes()
                    .filter(|node| excluded.binary_search(node).is_err())
                    .filter_map(|node| reachable.get(node).cloned().flatten())
                    .reduce(prefer_nontransaction);
                let reach = if site.targets().len() > 1 {
                    direct
                        .into_iter()
                        .chain(transitive)
                        .reduce(prefer_nontransaction)
                } else {
                    direct.or(transitive)
                };
                let Some(reach) = reach else {
                    continue;
                };
                let file = relative_slash_path(root, &site.file);
                if audit.usage.suppress(
                    options
                        .allow
                        .iter()
                        .map(|entry| allow::matches(entry, &file, site, position.line)),
                ) {
                    continue;
                }
                let mut path = reach.path.clone();
                if let Some(caller) = &site.caller {
                    path.insert(0, caller.clone());
                }
                let path = path.join(" → ");
                let transaction = if reach.transaction {
                    " (transaction client only; merge statements instead of parallelizing)"
                } else {
                    ""
                };
                let message = application.message.as_deref().unwrap_or("per-item call reaches a configured effect sink; batch the helper, merge the lookup, or add a justified suppression");
                findings.push((
                    position.offset,
                    RuleFinding {
                        causes: Vec::new(),
                        rule: RULE_ID.to_string(),
                        file,
                        line: position.line as usize,
                        message: format!(
                            "{message}: {} `{}` reaches `{kind}` via {path}{transaction}",
                            position.construct, site.source_callee
                        ),
                        import: Some(site.source_callee.clone()),
                        target: Some(format!("{kind}: {path}")),
                    },
                ));
            }
        }
        findings.extend(
            allow::finish(root, facts, sources, &filter, &options, &audit)
                .into_iter()
                .map(|finding| (0, finding)),
        );
        findings.extend(
            languages::collect(root, config, application, &options, &audit, session)?
                .into_iter()
                .map(|finding| (0, finding)),
        );
    }
    Ok(traces::finalize_findings(findings))
}
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod allow_tests;
#[cfg(test)]
mod tests;
