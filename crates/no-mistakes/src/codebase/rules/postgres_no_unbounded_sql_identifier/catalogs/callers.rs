use super::*;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::NodeId;

impl<'a> index::Index<'a> {
    pub(super) fn index_calls(&mut self, graph: &'a DepGraph, facts: &'a CheckFactMap) {
        for file in facts.graph_file_universe() {
            let Some(flow) = self.flows.get(file.as_path()) else {
                continue;
            };
            let by_offset: FxHashMap<_, _> =
                flow.calls.iter().map(|call| (call.offset, call)).collect();
            for site in graph.call_sites_in_file(file) {
                for target in site.targets() {
                    let Some(NodeId::Symbol {
                        file: owner_file,
                        callable_id: Some(owner),
                        ..
                    }) = target.node
                    else {
                        continue;
                    };
                    let helper = (owner_file.as_ref(), *owner);
                    if !self.accessors.contains_key(&helper) {
                        continue;
                    }
                    let Some(call) = by_offset.get(&site.offset) else {
                        self.bad_helpers.insert(helper);
                        continue;
                    };
                    if site.invocation != InvocationKind::Call
                        || site.has_unknown_target()
                        || site.targets().count() != 1
                        || !site.alternatives.is_empty()
                        || graph.failed_call_sources(site).next().is_some()
                    {
                        self.bad_helpers.insert(helper);
                    }
                    self.incoming.entry(helper).or_default().push((
                        file.as_path(),
                        call.offset,
                        call.end,
                    ));
                }
            }
        }
    }
}
