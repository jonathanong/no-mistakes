use super::super::config::Invocation;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite};

pub(super) fn allowed_invocations(configured: &[Invocation]) -> Vec<InvocationKind> {
    let values = configured
        .iter()
        .map(|kind| match kind {
            Invocation::Call => InvocationKind::Call,
            Invocation::Construct => InvocationKind::Construct,
        })
        .collect::<Vec<_>>();
    if values.is_empty() {
        vec![InvocationKind::Call]
    } else {
        values
    }
}

pub(super) fn site_source_node(site: &ResolvedCallSite) -> NodeId {
    site.caller.as_deref().map_or_else(
        || NodeId::file(&site.file),
        |symbol| {
            site.caller_id.map_or_else(
                || NodeId::symbol(&site.file, symbol),
                |id| NodeId::callable(&site.file, symbol.to_string(), id),
            )
        },
    )
}
