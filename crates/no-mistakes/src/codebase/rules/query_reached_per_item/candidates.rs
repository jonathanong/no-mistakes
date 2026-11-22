use super::{CheckFactMap, DepGraph, ResolvedCallSite, TsFactLookup};
use crate::codebase::dependencies::extract::{InvocationKind, PerItemCall};
use crate::codebase::rules::path_filter::RulePathFilter;
use std::collections::BTreeMap;

pub(super) fn select<'a>(
    graph: &'a DepGraph,
    facts: &'a CheckFactMap,
    filter: &RulePathFilter,
) -> Vec<(&'a ResolvedCallSite, &'a PerItemCall)> {
    let observer = crate::diagnostics::current();
    let mut positions_examined = 0;
    let mut sites_examined = 0;
    let mut selected = Vec::new();
    for file in facts.graph_file_universe() {
        let Some(ts) = facts.get_ts_facts(file) else {
            continue;
        };
        if ts.per_item_calls.is_empty()
            || ts.parse_error.is_some()
            || ts.operational_error.is_some()
            || !filter.is_match(file)
        {
            continue;
        }
        let sites = graph.call_sites_in_file(file);
        let mut positions = BTreeMap::new();
        for position in &ts.per_item_calls {
            positions.insert(
                (position.offset, position.caller_id, position.is_callback),
                position,
            );
        }
        if observer.is_some() {
            positions_examined += positions.len() as u64;
        }
        for ((offset, caller, callback), position) in positions {
            let start = sites.partition_point(|site| site.offset < offset);
            let count = sites[start..].partition_point(|site| site.offset == offset);
            if observer.is_some() {
                sites_examined += count as u64;
            }
            selected.extend(
                sites[start..start + count]
                    .iter()
                    .filter(|site| {
                        site.caller_id == caller
                            && (site.invocation == InvocationKind::Callback) == callback
                    })
                    .map(|site| (site, position)),
            );
        }
    }
    if let Some(observer) = observer {
        observer.increment(
            "query_reached_per_item.candidate_positions",
            positions_examined,
        );
        observer.increment("query_reached_per_item.candidate_sites", sites_examined);
    }
    selected
}
