use super::*;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::NodeId;

pub(super) fn returned_builder(proof: &mut Proof<'_, '_>, file: &Path, call: &ValueCall) -> bool {
    if call.bound || call.spread_at.is_some() {
        return false;
    }
    let sites: Vec<_> = proof
        .graph
        .call_sites_in_file(file)
        .iter()
        .filter(|site| {
            site.offset == call.offset
                && site.caller_id == call.context
                && site.invocation == InvocationKind::Call
        })
        .collect();
    let [site] = sites.as_slice() else {
        return false;
    };
    if site.has_unknown_target() || proof.graph.failed_call_sources(site).next().is_some() {
        return false;
    }
    let targets: Vec<_> = site.targets().collect();
    let [target] = targets.as_slice() else {
        return false;
    };
    let Some(NodeId::Symbol {
        file,
        callable_id: Some(owner),
        ..
    }) = target.node
    else {
        return false;
    };
    let producer = file.as_ref();
    let Some(flow) = proof.file(producer).map(|data| data.flow) else {
        return false;
    };
    if flow.async_callables.contains(owner) || flow.generator_callables.contains(owner) {
        return false;
    }
    let returns: Vec<_> = flow
        .returns
        .iter()
        .filter(|(returned, _)| returned == owner)
        .collect();
    !returns.is_empty()
        && returns
            .iter()
            .all(|(_, value)| proof.returned(producer, *owner, value))
}
