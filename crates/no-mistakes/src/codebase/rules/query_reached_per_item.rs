//! Per-item policies project parser-owned positions and canonical call traces.
use super::RuleFinding;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

mod config;
mod traces;
pub(crate) use config::graph_plan;
use config::{Options, Sinks, exempt};
use traces::{Reach, source_node};

pub const RULE_ID: &str = "query-reached-per-item";

pub(crate) fn check_with_graph(
    root: &Path,
    config: &NoMistakesConfig,
    graph: &DepGraph,
    facts: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let root = root.as_path();
    // Occurrences have exact offsets, including multiple calls on one line.
    let mut positions = BTreeMap::new();
    for file in facts.graph_file_universe() {
        let Some(ts) = facts.get_ts_facts(file) else {
            continue;
        };
        if ts.parse_error.is_some() || ts.operational_error.is_some() {
            continue;
        }
        for position in &ts.per_item_calls {
            positions.insert(
                (
                    file.clone(),
                    position.offset,
                    position.caller_id,
                    position.is_callback,
                ),
                position,
            );
        }
    }
    let mut findings = Vec::new();
    for application in config.rule_applications(RULE_ID) {
        let options: Options = application.try_rule_options()?;
        config::validate(&options, config)?;
        let filter = super::path_filter::RulePathFilter::new(root, config, application)?;
        for kind in &options.effects {
            let effect = &config.effects[kind];
            let configured_sinks = Sinks::new(effect, graph);
            let mut sinks: BTreeMap<NodeId, Vec<(&ResolvedCallSite, bool)>> = BTreeMap::new();
            let mut excluded = Vec::new();
            for site in graph.resolved_call_sites() {
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
            let candidates = graph
                .resolved_call_sites()
                .iter()
                .filter_map(|site| {
                    // A source name can refer to different lexical bindings in
                    // one caller. Correlate each resolved callback by its use offset.
                    positions
                        .get(&(
                            site.file.clone(),
                            site.offset,
                            site.caller_id,
                            site.invocation
                                == crate::codebase::dependencies::extract::InvocationKind::Callback,
                        ))
                        .map(|position| (site, *position))
                })
                .collect::<Vec<_>>();
            let mut roots = candidates
                .iter()
                .flat_map(|(site, _)| site.target_nodes().cloned())
                .collect::<Vec<_>>();
            roots.sort();
            roots.dedup();
            roots.retain(|node| excluded.binary_search(node).is_err());
            let reachable = traces::reachable(graph, &roots, &sinks, &excluded);
            for (site, position) in candidates {
                if !filter.is_match(&site.file)
                    || exempt(effect, &site.source_callee)
                    || excluded.binary_search(&source_node(site)).is_ok()
                    || (site.targets().len() > 0
                        && site.targets().all(|candidate| {
                            candidate
                                .node
                                .is_some_and(|node| excluded.binary_search(node).is_ok())
                        }))
                {
                    continue;
                }
                let direct = configured_sinks.match_site(site).map(|transaction| Reach {
                    path: vec![site.source_callee.clone()],
                    transaction,
                });
                let transitive = site
                    .target_nodes()
                    .filter(|node| excluded.binary_search(node).is_err())
                    .filter_map(|node| reachable.get(node).cloned().flatten())
                    .reduce(prefer_nontransaction);
                let reach = if matches!(
                    &site.target,
                    crate::codebase::dependencies::graph::ResolvedCallTarget::Alternatives(_)
                ) {
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
                if options.allow.iter().any(|allow| {
                    allow.file == file
                        && allow
                            .callee
                            .as_ref()
                            .is_none_or(|callee| callee == &site.source_callee)
                        && allow.line.is_none_or(|line| line == position.line)
                }) {
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
    }
    Ok(traces::finalize_findings(findings))
}

fn prefer_nontransaction(left: Reach, right: Reach) -> Reach {
    if left.transaction && !right.transaction {
        right
    } else {
        left
    }
}

#[cfg(test)]
mod tests;
