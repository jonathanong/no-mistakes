use super::*;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::NodeId;

pub(super) fn returned_builder(proof: &mut Proof<'_, '_>, call: &ValueCall) -> bool {
    if call.bound || call.spread_at.is_some() {
        return false;
    }
    let sites: Vec<_> = proof
        .graph
        .call_sites_in_file(proof.file)
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
    if file.as_ref() != proof.file
        || proof.flow.async_callables.contains(owner)
        || proof.flow.generator_callables.contains(owner)
    {
        return false;
    }
    let returns: Vec<_> = proof
        .flow
        .returns
        .iter()
        .filter(|(returned, _)| returned == owner)
        .collect();
    !returns.is_empty()
        && returns
            .iter()
            .all(|(_, value)| proof.expression(value, true))
}
