use crate::codebase::dependencies::graph::{CallTraversal, DepGraph, NodeId, ResolvedCallSite};
use rayon::prelude::*;
use std::collections::BTreeMap;

pub(super) fn source_node(site: &ResolvedCallSite) -> NodeId {
    site.caller.as_deref().map_or_else(
        || NodeId::file(&site.file),
        |caller| {
            site.caller_id.map_or_else(
                || NodeId::symbol(&site.file, caller),
                |id| NodeId::callable(&site.file, caller.to_string(), id),
            )
        },
    )
}
pub(super) fn node_label(node: &NodeId) -> String {
    match node {
        NodeId::Symbol { symbol, .. } => symbol.to_string(),
        _ => "module".to_string(),
    }
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Reach {
    pub(super) path: Vec<String>,
    pub(super) transaction: bool,
}
pub(super) fn shortest(values: impl Iterator<Item = Reach>) -> Option<Reach> {
    values.min_by(|left, right| {
        (left.path.len(), &left.path, left.transaction).cmp(&(
            right.path.len(),
            &right.path,
            right.transaction,
        ))
    })
}

pub(super) type SinkMap<'a> = BTreeMap<NodeId, Vec<(&'a ResolvedCallSite, bool)>>;
pub(super) fn reachable(
    graph: &DepGraph,
    roots: &[NodeId],
    sinks: &SinkMap<'_>,
    excluded: &[NodeId],
) -> BTreeMap<NodeId, Option<Reach>> {
    roots
        .par_iter()
        .map(|node| {
            let mut paths = vec![(node.clone(), vec![node.clone()])];
            paths.extend(
                graph
                    .call_traces_excluding(
                        std::slice::from_ref(node),
                        CallTraversal::Transitive,
                        None,
                        excluded,
                    )
                    .into_iter()
                    .map(|trace| (trace.target, trace.nodes)),
            );
            let values = paths
                .into_iter()
                .flat_map(|(target, path)| {
                    sinks
                        .get(&target)
                        .into_iter()
                        .flatten()
                        .map(move |(site, transaction)| {
                            let mut labels = path.iter().map(node_label).collect::<Vec<_>>();
                            labels.push(site.source_callee.clone());
                            Reach {
                                path: labels,
                                transaction: *transaction,
                            }
                        })
                })
                .collect::<Vec<_>>();
            // Only-transaction classification examines all reached sinks,
            // not just whichever shortest trace happens to win a tie.
            let only_transaction =
                !values.is_empty() && values.iter().all(|value| value.transaction);
            let best = shortest(values.into_iter()).map(|mut value| {
                value.transaction = only_transaction;
                value
            });
            (node.clone(), best)
        })
        .collect()
}

/// Keep repeated physical calls on the same line distinct while deduplicating
/// an occurrence selected by overlapping rule applications.
pub(super) fn finalize_findings(
    mut findings: Vec<(u32, super::RuleFinding)>,
) -> Vec<super::RuleFinding> {
    findings.sort();
    findings.dedup();
    let mut totals = BTreeMap::new();
    for (_, finding) in &findings {
        *totals.entry(finding.clone()).or_insert(0usize) += 1;
    }
    let mut seen = BTreeMap::new();
    let mut findings = findings
        .into_iter()
        .map(|(_, mut finding)| {
            if totals.get(&finding).copied().unwrap_or_default() > 1 {
                let occurrence = seen.entry(finding.clone()).or_insert(0usize);
                *occurrence += 1;
                finding.message = format!("{} (occurrence {})", finding.message, occurrence);
            }
            finding
        })
        .collect::<Vec<_>>();
    findings.sort();
    findings
}
