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
use config::{exempt, sink, Options};
use traces::{source_node, Reach};

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
    let mut callbacks = BTreeMap::new();
    for file in facts.graph_file_universe() {
        let Some(ts) = facts.get_ts_facts(file) else {
            continue;
        };
        if ts.parse_error.is_some() || ts.operational_error.is_some() {
            continue;
        }
        for position in &ts.per_item_calls {
            if position.is_callback {
                callbacks
                    .entry((file.clone(), position.caller_id, position.callee.clone()))
                    .or_insert_with(Vec::new)
                    .push(position);
            } else {
                positions.insert(
                    (file.clone(), position.offset, position.caller_id),
                    position,
                );
            }
        }
    }
    let mut findings = Vec::new();
    for application in config.rule_applications(RULE_ID) {
        let options: Options = application.try_rule_options()?;
        config::validate(&options, config)?;
        let filter = super::path_filter::RulePathFilter::new(root, config, application)?;
        for kind in &options.effects {
            let effect = &config.effects[kind];
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
                    if let Some(node) = &site.target_node {
                        excluded.push(node.clone());
                    }
                } else if let Some(transaction) = sink(effect, &site.source_callee) {
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
                    let position = if site.invocation
                        == crate::codebase::dependencies::extract::InvocationKind::Callback
                    {
                        callbacks
                            .get(&(
                                site.file.clone(),
                                site.caller_id,
                                site.source_callee.clone(),
                            ))
                            .map(Vec::as_slice)
                    } else {
                        positions
                            .get(&(site.file.clone(), site.offset, site.caller_id))
                            .map(std::slice::from_ref)
                    };
                    position.map(|positions| (site, positions))
                })
                .collect::<Vec<_>>();
            let mut roots = candidates
                .iter()
                .filter_map(|(site, _)| site.target_node.clone())
                .collect::<Vec<_>>();
            roots.sort();
            roots.dedup();
            roots.retain(|node| excluded.binary_search(node).is_err());
            let reachable = traces::reachable(graph, &roots, &sinks, &excluded);
            for (site, positions) in candidates {
                if !filter.is_match(&site.file)
                    || exempt(effect, &site.source_callee)
                    || excluded.binary_search(&source_node(site)).is_ok()
                    || site
                        .target_node
                        .as_ref()
                        .is_some_and(|node| excluded.binary_search(node).is_ok())
                {
                    continue;
                }
                let direct = sink(effect, &site.source_callee).map(|transaction| Reach {
                    path: vec![site.source_callee.clone()],
                    transaction,
                });
                let reach = direct.or_else(|| {
                    site.target_node
                        .as_ref()
                        .and_then(|node| reachable.get(node))
                        .cloned()
                        .flatten()
                });
                let Some(reach) = reach else {
                    continue;
                };
                for position in positions {
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
                            source_offset: None,
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
    }
    Ok(traces::finalize_findings(findings))
}

#[cfg(test)]
mod tests;
