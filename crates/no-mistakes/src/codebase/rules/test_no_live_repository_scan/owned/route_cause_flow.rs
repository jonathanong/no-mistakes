//! Whether a judged parameter is an argument of a recorded call.
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::forbidden_calls::site_source_node;
use crate::fx::FxHashMap;

/// `Some` when this call's arguments were recorded. `None` when they were not.
pub(super) fn parameter_flows(
    callers: &FxHashMap<NodeId, Vec<&ResolvedCallSite>>,
    source: &NodeId,
    target: &NodeId,
    facts: &dyn TsFactLookup,
    index: usize,
) -> Option<bool> {
    let span = callable_span(source)?;
    let sites = callers.get(target)?;
    let mut saw = false;
    let mut matched = false;
    for site in sites
        .iter()
        .filter(|site| site_source_node(site) == *source)
    {
        let Some(paths) = facts
            .get_ts_facts(&site.file)
            .and_then(super::super::path_provenance)
        else {
            continue;
        };
        let proven = paths.arg_paths.get(&site.offset);
        let unproven = paths.arg_unproven.get(&site.offset);
        if proven.is_none() && unproven.is_none() {
            continue;
        }
        saw = true;
        matched |=
            carries_parameter(proven, index, span) || carries_parameter(unproven, index, span);
    }
    saw.then_some(matched)
}

fn carries_parameter(
    args: Option<
        &Vec<(
            usize,
            crate::codebase::ts_source::module_facts::arguments::PathOrigin,
        )>,
    >,
    index: usize,
    span: u32,
) -> bool {
    args.is_some_and(|args| {
        args.iter()
            .any(|(_, origin)| origin.param == Some(index) && origin.param_owner == Some(span))
    })
}

fn callable_span(source: &NodeId) -> Option<u32> {
    match source {
        NodeId::Symbol {
            callable_id: Some(id),
            ..
        } => Some(id.0),
        _ => None,
    }
}
