//! A const object of functions reached only through a member call.
#[path = "registry_alias.rs"]
mod alias;

use super::super::forbidden_calls::site_source_node;
use super::super::path_filter::RulePathFilter;
use super::config::Enumerator;
use super::owned::path_provenance;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{
    CallTraversal, DepGraph, ResolvedCallSite, TsFactLookup,
};

pub(super) fn apply<'a>(
    graph: &'a DepGraph,
    facts: &dyn TsFactLookup,
    scope: &mut super::reachable::Scope,
    filter: &RulePathFilter,
    enumerators: &[Enumerator],
) -> Vec<(&'a ResolvedCallSite, String)> {
    let callers = alias::caller_nodes(graph);
    let mut reached = Vec::new();
    let mut unresolved = Vec::new();
    for site in graph.resolved_call_sites() {
        if site.invocation != InvocationKind::Call
            || site.target_node.is_some()
            || !scope.sources.contains(&site_source_node(site))
            || !filter.is_match(&site.file)
        {
            continue;
        }
        let Some((object, key, label)) = dispatch(facts, site) else {
            continue;
        };
        let resolved = key
            .as_deref()
            .and_then(|key| alias::function_node(graph, facts, site, &object, key, &callers));
        if let Some(node) = resolved {
            reached.push(node);
            continue;
        }
        if key.is_none()
            && alias::registry_scans(graph, facts, site, &object, enumerators, &callers)
        {
            unresolved.push((site, label));
        }
    }
    if !reached.is_empty() {
        let extra = graph.call_source_nodes(&reached, CallTraversal::Transitive, None);
        scope.sources.extend(extra);
    }
    unresolved
}

/// `object.member`, or a computed call recorded while the file was parsed.
fn dispatch(
    facts: &dyn TsFactLookup,
    site: &ResolvedCallSite,
) -> Option<(String, Option<String>, String)> {
    if let Some((object, key)) = static_member(&site.source_callee) {
        return Some((object, key, String::new()));
    }
    let provenance = path_provenance(facts.get_ts_facts(&site.file)?)?;
    provenance
        .registry_dispatches
        .iter()
        .find(|dispatch| dispatch.0 == site.offset)
        .map(|dispatch| (dispatch.1.clone(), dispatch.2.clone(), dispatch.3.clone()))
}

fn static_member(callee: &str) -> Option<(String, Option<String>)> {
    let (object, key) = callee.split_once('.')?;
    (is_name(object) && is_name(key)).then(|| (object.to_string(), Some(key.to_string())))
}

fn is_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c == '_' || c == '$' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c == '$' || c.is_ascii_alphanumeric())
}
