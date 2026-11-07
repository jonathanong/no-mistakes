use super::*;
use crate::codebase::dependencies::extract::{CallableId, InvocationKind};
use crate::codebase::dependencies::graph::NodeId;

#[derive(Default)]
pub(super) struct State {
    index: Option<FxHashMap<CallableId, Vec<usize>>>,
    active: FxHashSet<ProofKey>,
    pub(super) done: FxHashMap<ProofKey, bool>,
}

#[path = "text_tag_origins.rs"]
mod origins;

pub(super) fn parameter_bound(
    bindings: &Bindings<'_>,
    file: &Path,
    parameter: &ValueParameter,
    index: usize,
    members: &[String],
    tags: &[TrustedSqlTag],
    state: &mut ProofState,
) -> bool {
    if state.opaque_parameter_defaults.contains(&index) {
        return false;
    }
    let key = ProofKey::Parameter(parameter.scope, parameter.name.clone(), members.to_vec());
    origin_memoized(state, key, |state| {
        if !arguments::parameter_defaults(bindings, file, parameter, members, tags, state) {
            return false;
        }
        let Some(graph) = bindings.graph else {
            return true;
        };
        let Some(ts) = bindings.facts.get_ts_facts(file) else {
            return false;
        };
        if state.callers.index.is_none() {
            state.callers.index = Some(caller_index(graph, file, &ts.value_flow));
        }
        let calls = state
            .callers
            .index
            .as_ref()
            .and_then(|index| index.get(&parameter.owner))
            .cloned()
            .unwrap_or_default();
        let path: Vec<_> = parameter
            .projection
            .iter()
            .filter_map(|step| match step {
                ValueProjection::Member(member) | ValueProjection::Element(member) => {
                    Some(member.clone())
                }
                ValueProjection::Default(_, _) => None,
            })
            .chain(members.iter().cloned())
            .collect();
        calls.iter().all(|index| {
            let call = &ts.value_flow.calls[*index];
            if call
                .spread_at
                .is_some_and(|spread| spread <= parameter.index)
            {
                return false;
            }
            origins::caller_argument(bindings, file, call, parameter.index, &path, tags, state)
        })
    })
}

pub(super) fn origin_memoized(
    state: &mut ProofState,
    key: ProofKey,
    evaluate: impl FnOnce(&mut ProofState) -> bool,
) -> bool {
    if let Some(done) = state.callers.done.get(&key) {
        return *done;
    }
    if !state.callers.active.insert(key.clone()) {
        return false;
    }
    let result = evaluate(state);
    state.callers.active.remove(&key);
    state.callers.done.insert(key, result);
    result
}

fn caller_index(
    graph: &crate::codebase::dependencies::graph::DepGraph,
    file: &Path,
    flow: &crate::codebase::dependencies::extract::ValueFlowFacts,
) -> FxHashMap<CallableId, Vec<usize>> {
    let mut offsets: FxHashMap<u32, Vec<usize>> = FxHashMap::default();
    for (index, call) in flow
        .calls
        .iter()
        .enumerate()
        .filter(|(_, call)| !call.bound)
    {
        offsets.entry(call.offset).or_default().push(index);
    }
    let mut callers: FxHashMap<CallableId, Vec<usize>> = FxHashMap::default();
    for site in graph
        .call_sites_in_file(file)
        .iter()
        .filter(|site| site.invocation == InvocationKind::Call)
    {
        let Some(indices) = offsets.get(&site.offset) else {
            continue;
        };
        for target in site.targets() {
            let Some(NodeId::Symbol {
                file: producer,
                callable_id: Some(owner),
                ..
            }) = target.node
            else {
                continue;
            };
            if producer.as_ref() == file {
                callers.entry(*owner).or_default().extend(
                    indices
                        .iter()
                        .copied()
                        .filter(|index| flow.calls[*index].context == site.caller_id),
                );
            }
        }
    }
    for calls in callers.values_mut() {
        calls.sort_unstable();
        calls.dedup();
    }
    callers
}
